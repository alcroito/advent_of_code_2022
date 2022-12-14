extern crate derive_more;
use derive_more::Display;
use itertools::Itertools;
use std::path::Path;
use tailsome::IntoResult;

#[derive(Debug, Display)]
#[display(fmt = "{} (file, size={})", name, size)]
struct File {
    name: String,
    size: usize,
}

impl File {
    fn new(name: &str, size: usize) -> Self {
        Self {
            name: name.to_owned(),
            size,
        }
    }
}

#[derive(Debug, Display, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[display(fmt = "entry_idx: {}", _0)]
struct FSEntryIdx(usize);

impl FSEntryIdx {
    fn new(i: usize) -> Self {
        FSEntryIdx(i)
    }
}

type ChildrenIdx = Vec<FSEntryIdx>;

#[derive(Debug, Display)]
#[display(fmt = "{} (dir)", name)]
struct Directory {
    name: String,
    children: ChildrenIdx,
    index: Option<FSEntryIdx>,
    parent: Option<FSEntryIdx>,
    size: std::cell::Cell<Option<usize>>,
}

impl Directory {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            children: vec![],
            index: None,
            parent: None,
            size: std::cell::Cell::new(None),
        }
    }

    fn find_child_by_name(&self, name: &str, fs: &FSArena) -> Option<FSEntryIdx> {
        self.children
            .iter()
            .find(|idx| fs[**idx].name() == name)
            .cloned()
    }

    fn get_size(&self, fs: &FSArena) -> usize {
        if let Some(size) = self.size.get() {
            return size;
        }

        let size = fs
            .iter_dir_contents(self)
            .map(|(fs_entry, _)| match fs_entry {
                FSEntry::File(file) => file.size,
                FSEntry::Directory(directory) => directory.get_size(fs),
            })
            .sum();
        self.size.set(Some(size));
        size
    }
}

#[derive(Debug, Display)]
enum FSEntry {
    #[display(fmt = "{}", _0)]
    File(File),
    #[display(fmt = "{}", _0)]
    Directory(Directory),
}

impl FSEntry {
    fn find_child_idx_by_name(&self, name: &str, fs: &FSArena) -> Option<FSEntryIdx> {
        if let Self::Directory(dir) = self {
            return dir.find_child_by_name(name, fs);
        }
        None
    }

    fn name(&self) -> &str {
        match self {
            FSEntry::File(e) => &e.name,
            FSEntry::Directory(e) => &e.name,
        }
    }
}

#[derive(Debug)]
struct FSArena {
    entries: Vec<FSEntry>,
}

impl FSArena {
    fn new() -> Self {
        Self { entries: vec![] }
    }

    fn next_index(&self) -> FSEntryIdx {
        FSEntryIdx::new(self.entries.len())
    }

    fn root_index(&self) -> FSEntryIdx {
        FSEntryIdx::new(0)
    }

    fn root_directory(&self) -> Result<&Directory, Error> {
        self.entries
            .get(self.root_index().0)
            .ok_or(Error::NoRootDirectoryInFS)
            .and_then(|fs_entry| match fs_entry {
                FSEntry::Directory(dir) => dir.into_ok(),
                _ => Error::NoRootDirectoryInFS.into_err(),
            })
    }

    fn iter(&self) -> FSIter {
        FSIter {
            to_visit: vec![(self.root_index(), 0)],
            fs: self,
            recursive: true,
        }
    }

    fn iter_dir_contents(&self, dir: &Directory) -> FSIter {
        FSIter {
            to_visit: dir.children.iter().map(|e| (*e, 0)).collect(),
            fs: self,
            recursive: false,
        }
    }

    fn dir_iter(&self) -> DirectoryIter {
        DirectoryIter {
            to_visit: vec![self.root_index()],
            fs: self,
        }
    }
}

impl std::fmt::Display for FSArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.iter().try_for_each(|(fs_entry, rank)| {
            let indent = str::repeat(" ", rank * 2);
            writeln!(f, "{}- {}", indent, fs_entry)
        })
    }
}

impl std::ops::Index<FSEntryIdx> for FSArena {
    type Output = FSEntry;
    fn index(&self, index: FSEntryIdx) -> &FSEntry {
        &self.entries[index.0]
    }
}

impl std::ops::IndexMut<FSEntryIdx> for FSArena {
    fn index_mut(&mut self, index: FSEntryIdx) -> &mut Self::Output {
        &mut self.entries[index.0]
    }
}

struct DirectoryIter<'a> {
    to_visit: ChildrenIdx,
    fs: &'a FSArena,
}

impl<'a> Iterator for DirectoryIter<'a> {
    type Item = &'a Directory;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let idx = self.to_visit.pop()?;
            let fs_entry = &self.fs[idx];
            if let FSEntry::Directory(directory) = fs_entry {
                self.to_visit.extend(directory.children.iter().copied());
                return Some(directory);
            }
        }
    }
}

struct FSIter<'a> {
    to_visit: Vec<(FSEntryIdx, usize)>,
    fs: &'a FSArena,
    recursive: bool,
}

impl<'a> Iterator for FSIter<'a> {
    type Item = (&'a FSEntry, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, rank) = self.to_visit.pop()?;
        let fs_entry = &self.fs[idx];
        if self.recursive {
            if let FSEntry::Directory(directory) = fs_entry {
                self.to_visit
                    .extend(directory.children.iter().map(|e| (*e, rank + 1)));
            }
        }
        Some((fs_entry, rank))
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", name)]
struct ChangeDirectoryArg {
    name: String,
}

impl ChangeDirectoryArg {
    fn new(s: &str) -> Self {
        Self { name: s.to_owned() }
    }
}

#[derive(Debug, Display)]
enum Op {
    #[display(fmt = "$ cd {}", _0)]
    ChangeDirectory(ChangeDirectoryArg),
    #[display(fmt = "$ cd ..")]
    ChangeDirectoryUp,
    #[display(fmt = "$ cd /")]
    ChangeDirectoryRoot,
    #[display(fmt = "$ ls")]
    List,
}

#[derive(Debug, Display)]
enum ParsedEntry {
    #[display(fmt = "{}", _0)]
    DoOp(Op),
    #[display(fmt = "{}", _0)]
    ListFSEntry(FSEntry),
}

type ParsedEntries = Vec<ParsedEntry>;

fn parse_op(l: &str) -> Result<Op, Error> {
    match l.split(' ').collect::<Vec<_>>().as_slice() {
        ["$", "cd", "/"] => Op::ChangeDirectoryRoot,
        ["$", "cd", ".."] => Op::ChangeDirectoryUp,
        ["$", "cd", dir] => Op::ChangeDirectory(ChangeDirectoryArg::new(dir)),
        ["$", "ls"] => Op::List,
        _ => Error::InvalidOperation(l.to_owned()).into_err()?,
    }
    .into_ok()
}

fn parse_fs_entry(l: &str) -> Result<FSEntry, Error> {
    match l.split(' ').collect::<Vec<_>>().as_slice() {
        ["dir", name] => FSEntry::Directory(Directory::new(name)),
        [size, name] => {
            let size = size.parse::<usize>()?;
            FSEntry::File(File::new(name, size))
        }
        _ => Error::InvalidFSEntry(l.to_owned()).into_err()?,
    }
    .into_ok()
}

fn parse_ops_and_fs_entries(s: &str) -> Result<ParsedEntries, Error> {
    s.split('\n')
        .map(|l| {
            match &l.get(0..1) {
                Some("$") => ParsedEntry::DoOp(parse_op(l)?),
                Some(_) => ParsedEntry::ListFSEntry(parse_fs_entry(l)?),
                None => Error::InvalidLine(l.to_owned()).into_err()?,
            }
            .into_ok()
        })
        .collect::<Result<Vec<ParsedEntry>, Error>>()
}

struct AssembleFSState {
    curr_dir_idx: FSEntryIdx,
    fs: FSArena,
}

fn handle_parsed_entry(
    entry: ParsedEntry,
    mut state: AssembleFSState,
) -> Result<AssembleFSState, Error> {
    let curr_dir_idx = &mut state.curr_dir_idx;
    let fs = &mut state.fs;
    match entry {
        ParsedEntry::DoOp(Op::ChangeDirectory(directory)) => {
            let curr_dir = &fs[*curr_dir_idx];
            let maybe_child_dir_idx = curr_dir.find_child_idx_by_name(&directory.name, fs);
            let child_dir_idx = maybe_child_dir_idx
                .ok_or_else(|| Error::FSEntryNotFound(directory.name.to_owned()))?;
            *curr_dir_idx = child_dir_idx;
        }
        ParsedEntry::DoOp(Op::ChangeDirectoryUp) => {
            let curr_dir = &fs[*curr_dir_idx];
            match curr_dir {
                FSEntry::Directory(curr_dir) => {
                    *curr_dir_idx = curr_dir
                        .parent
                        .ok_or_else(|| Error::DirHasNoParent(curr_dir.name.to_owned()))?
                }
                _ => Error::CurrentDirIsNotDir(curr_dir_idx.0).into_err()?,
            }
        }
        ParsedEntry::DoOp(Op::ChangeDirectoryRoot) => {
            *curr_dir_idx = fs.next_index();
            let mut directory = Directory::new("/");
            directory.index = Some(fs.root_index());
            let new_dir = FSEntry::Directory(directory);
            fs.entries.push(new_dir);
        }
        ParsedEntry::DoOp(Op::List) => {
            // TODO: Validate that next ParsedEntry is an FSEntry
        }
        ParsedEntry::ListFSEntry(FSEntry::File(file)) => {
            let curr_dir = &fs[*curr_dir_idx];
            assert!(curr_dir.find_child_idx_by_name(&file.name, fs).is_none());
            let new_file = FSEntry::File(file);
            let new_file_idx = fs.next_index();
            fs.entries.push(new_file);

            // Reborrow as mutable
            let curr_dir = &mut fs[*curr_dir_idx];
            match curr_dir {
                FSEntry::Directory(curr_dir) => curr_dir.children.push(new_file_idx),
                _ => Error::CurrentDirIsNotDir(curr_dir_idx.0).into_err()?,
            }
        }
        ParsedEntry::ListFSEntry(FSEntry::Directory(directory)) => {
            let curr_dir = &fs[*curr_dir_idx];
            assert!(curr_dir
                .find_child_idx_by_name(&directory.name, fs)
                .is_none());
            let mut directory = directory;
            let new_dir_idx = fs.next_index();
            directory.parent = Some(*curr_dir_idx);
            directory.index = Some(new_dir_idx);
            let new_dir = FSEntry::Directory(directory);
            fs.entries.push(new_dir);

            // Reborrow as mutable
            let curr_dir = &mut fs[*curr_dir_idx];
            match curr_dir {
                FSEntry::Directory(curr_dir) => curr_dir.children.push(new_dir_idx),
                _ => Error::CurrentDirIsNotDir(curr_dir_idx.0).into_err()?,
            }
        }
    };
    Ok(state)
}

fn assemble_fs(entries: ParsedEntries) -> Result<FSArena, Error> {
    let state = AssembleFSState {
        curr_dir_idx: FSEntryIdx::new(0),
        fs: FSArena::new(),
    };
    entries
        .into_iter()
        .try_fold(state, |state, e| handle_parsed_entry(e, state))
        .map(|state| state.fs)
}

fn sum_dirs_with_size_lt_100k(fs: &FSArena) -> usize {
    fs.dir_iter()
        .map(|dir| dir.get_size(fs))
        .filter(|size| *size < 100_000)
        .sum()
}

fn find_smallest_dir_to_del(fs: &FSArena) -> Result<usize, Error> {
    let total_space: usize = 70_000_000;
    let needed_space: usize = 30_000_000;
    let used_space = fs.root_directory()?.get_size(fs);
    fs.dir_iter()
        .map(|dir| dir.get_size(fs))
        .sorted()
        .find(|size| (total_space - used_space + size) > needed_space)
        .ok_or(Error::NoSolution)
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let entries = parse_ops_and_fs_entries(&s)?;
    let fs = assemble_fs(entries)?;
    let res = sum_dirs_with_size_lt_100k(&fs);
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let entries = parse_ops_and_fs_entries(&s)?;
    let fs = assemble_fs(entries)?;
    let res = find_smallest_dir_to_del(&fs)?;
    println!("p2: {}", res);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
    #[error("Invalid operation {0}")]
    InvalidOperation(String),
    #[error("Invalid file system entry {0}")]
    InvalidFSEntry(String),
    #[error("Failed to parse line {0}")]
    InvalidLine(String),
    #[error(transparent)]
    InvalidFileSize(#[from] std::num::ParseIntError),
    #[error("No root directory found")]
    NoRootDirectoryInFS,
    #[error("File system entry not found {0}")]
    FSEntryNotFound(String),
    #[error("Directory has no parent {0}")]
    DirHasNoParent(String),
    #[error("PWD is not a directory {0}")]
    CurrentDirIsNotDir(usize),
}
