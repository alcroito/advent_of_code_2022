extern crate derive_more;
use derive_more::Display;
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
    parent: Option<FSEntryIdx>,
    size: std::cell::Cell<Option<usize>>,
}

impl Directory {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            children: vec![],
            parent: None,
            size: std::cell::Cell::new(None),
        }
    }

    fn find_child_by_name(&self, name: &str, fs: &FSArena) -> Option<FSEntryIdx> {
        self.children
            .iter()
            .find(|idx| {
                let entry = &fs[**idx];
                match entry {
                    FSEntry::File(file) => {
                        if file.name == name {
                            return true;
                        }
                    }
                    FSEntry::Directory(dir) => {
                        if dir.name == name {
                            return true;
                        }
                    }
                }
                false
            })
            .cloned()
    }

    fn get_size(&self, fs: &FSArena) -> usize {
        if let Some(size) = self.size.get() {
            return size;
        }

        let mut size = 0;
        for idx in &self.children {
            let fs_entry = &fs[*idx];

            match fs_entry {
                FSEntry::File(file) => size += file.size,
                FSEntry::Directory(directory) => size += directory.get_size(fs),
            }
        }
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
}

impl std::fmt::Display for FSArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent_lvl = 0;
        let mut to_visit = vec![(indent_lvl, self.root_index())];
        while !to_visit.is_empty() {
            let (indent_lvl, idx) = to_visit.pop().expect("Expected element in stack");
            let indent = str::repeat(" ", indent_lvl * 2);
            let fs_entry = &self[idx];
            writeln!(f, "{}- {}", indent, fs_entry)?;

            if let FSEntry::Directory(directory) = fs_entry {
                to_visit.extend(directory.children.iter().map(|e| (indent_lvl + 1, *e)))
            }
        }
        Ok(())
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

fn assemble_fs(entries: ParsedEntries) -> FSArena {
    let mut fs = FSArena::new();
    let mut curr_dir_idx = FSEntryIdx::new(0);
    for entry in entries {
        match entry {
            ParsedEntry::DoOp(op) => {
                match op {
                    Op::ChangeDirectory(directory) => {
                        let curr_dir = &fs[curr_dir_idx];
                        let maybe_child_dir_idx =
                            curr_dir.find_child_idx_by_name(&directory.name, &fs);
                        // FIXME use Result
                        let child_dir_idx =
                            maybe_child_dir_idx.expect("Directory should have been created");
                        curr_dir_idx = child_dir_idx;
                    }
                    Op::ChangeDirectoryUp => {
                        let curr_dir = &fs[curr_dir_idx];
                        if let FSEntry::Directory(curr_dir) = curr_dir {
                            // FIXME use Result
                            curr_dir_idx = curr_dir
                                .parent
                                .expect("Expecting directory to have a parent")
                        } else {
                            // FIXME use Result
                            unreachable!("Expected to find a directory")
                        }
                    }
                    Op::ChangeDirectoryRoot => {
                        curr_dir_idx = fs.next_index();
                        let new_dir = FSEntry::Directory(Directory::new("/"));
                        fs.entries.push(new_dir);
                    }
                    Op::List => {
                        // TODO: Validate that next ParsedEntry is an FSEntry
                    }
                }
            }
            ParsedEntry::ListFSEntry(fs_entry) => {
                match fs_entry {
                    FSEntry::File(file) => {
                        let curr_dir = &fs[curr_dir_idx];
                        assert!(curr_dir.find_child_idx_by_name(&file.name, &fs).is_none());
                        let new_file = FSEntry::File(file);
                        let new_file_idx = fs.next_index();
                        fs.entries.push(new_file);
                        let curr_dir = &mut fs[curr_dir_idx];
                        if let FSEntry::Directory(curr_dir) = curr_dir {
                            curr_dir.children.push(new_file_idx)
                        } else {
                            // FIXME use Result
                            unreachable!("Expected to find a file")
                        }
                    }
                    FSEntry::Directory(directory) => {
                        let curr_dir = &fs[curr_dir_idx];
                        assert!(curr_dir
                            .find_child_idx_by_name(&directory.name, &fs)
                            .is_none());
                        let mut directory = directory;
                        directory.parent = Some(curr_dir_idx);
                        let new_dir = FSEntry::Directory(directory);
                        let new_dir_idx = fs.next_index();
                        fs.entries.push(new_dir);
                        let curr_dir = &mut fs[curr_dir_idx];
                        if let FSEntry::Directory(curr_dir) = curr_dir {
                            curr_dir.children.push(new_dir_idx)
                        } else {
                            // FIXME use Result
                            unreachable!("Expected to find a directory")
                        }
                    }
                }
            }
        }
    }
    fs
}

fn sum_dirs_with_size_lt_100k(fs: &FSArena) -> usize {
    let mut to_visit = vec![fs.root_index()];
    let mut filtered = vec![];
    while !to_visit.is_empty() {
        let idx = to_visit.pop().expect("Expected element in stack");
        let fs_entry = &fs[idx];
        if let FSEntry::Directory(directory) = fs_entry {
            let dir_size = directory.get_size(fs);
            if dir_size < 100_000 {
                filtered.push(dir_size);
            }
            to_visit.extend(directory.children.iter().copied())
        }
    }
    filtered.into_iter().sum()
}

const TOTAL_SPACE: usize = 70_000_000;
const NEEDED_SPACE: usize = 30_000_000;

fn find_smallest_dir_to_del(fs: &FSArena) -> usize {
    let mut to_visit = vec![fs.root_index()];
    let mut sizes = vec![];
    while !to_visit.is_empty() {
        let idx = to_visit.pop().expect("Expected element in stack");
        let fs_entry = &fs[idx];
        if let FSEntry::Directory(directory) = fs_entry {
            let dir_size = directory.get_size(fs);
            sizes.push(dir_size);
            to_visit.extend(directory.children.iter().copied())
        }
    }
    sizes.sort();
    let used_space = if let FSEntry::Directory(dir) = &fs[fs.root_index()] {
        dir.get_size(fs)
    } else {
        unreachable!("Root directory should exist");
    };
    sizes
        .into_iter()
        .find(|size| (TOTAL_SPACE - used_space + size) > NEEDED_SPACE)
        .expect("Expected to find result")
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let entries = parse_ops_and_fs_entries(&s)?;
    let fs = assemble_fs(entries);
    let res = sum_dirs_with_size_lt_100k(&fs);
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let entries = parse_ops_and_fs_entries(&s)?;
    let fs = assemble_fs(entries);
    let res = find_smallest_dir_to_del(&fs);
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
}
