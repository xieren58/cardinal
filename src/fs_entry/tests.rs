use super::*;
use pathbytes::b2p;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::{
    fs::{self, File},
    os::unix::fs as unixfs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// Compare two entries without comparing the create, accessed, modified time.
/// Useful for manually testing.
fn compare_test_entry(a: impl Borrow<DiskEntry>, b: impl Borrow<DiskEntry>) {
    let a = a.borrow();
    let b = b.borrow();
    if a.name != b.name {
        panic!("a: {:?} b: {:?}", b2p(&a.name), b2p(&b.name))
    }
    if a.metadata.clone().map(|Metadata { file_type, len, .. }| {
        (file_type, if file_type == FileType::File { len } else { 0 })
    }) != b.metadata.clone().map(|Metadata { file_type, len, .. }| {
        (file_type, if file_type == FileType::File { len } else { 0 })
    }) {
        panic!("a: {:?} b: {:?}", b2p(&a.name), b2p(&b.name))
    }
    let a_set: BTreeSet<_> = a.entries.clone().into_iter().collect();
    let b_set: BTreeSet<_> = b.entries.clone().into_iter().collect();
    a_set
        .into_iter()
        .zip(b_set.into_iter())
        .for_each(|(a, b)| compare_test_entry(a, b))
}

fn complex_entry<P: AsRef<Path>>(path: P) -> DiskEntry {
    DiskEntry {
        name: p2b(path.as_ref()).to_vec(),
        metadata: Some(Metadata {
            file_type: FileType::Dir,
            ..Default::default()
        }),
        entries: vec![
            DiskEntry {
                name: b"afolder".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::Dir,
                    ..Default::default()
                }),
                entries: vec![DiskEntry {
                    name: b"hello.txt".to_vec(),
                    metadata: Some(Metadata {
                        file_type: FileType::File,
                        len: 666,
                        ..Default::default()
                    }),
                    entries: Vec::new(),
                }],
            },
            DiskEntry {
                name: b"233.txt".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 233,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
            DiskEntry {
                name: "445.txt".into(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 445,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
            DiskEntry {
                name: "heck.txt".into(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 0,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
            DiskEntry {
                name: "src".into(),
                metadata: Some(Metadata {
                    file_type: FileType::Dir,
                    ..Default::default()
                }),
                entries: vec![DiskEntry {
                    name: "template".into(),
                    metadata: Some(Metadata {
                        file_type: FileType::Dir,
                        ..Default::default()
                    }),
                    entries: vec![DiskEntry {
                        name: "hello.java".into(),
                        metadata: Some(Metadata {
                            file_type: FileType::File,
                            len: 514,
                            ..Default::default()
                        }),
                        entries: Vec::new(),
                    }],
                }],
            },
        ],
    }
}

fn apply_complex_entry(path: &Path) {
    fs::create_dir_all(path.join("afolder")).unwrap();
    fs::write(path.join("afolder/hello.txt"), vec![42; 666]).unwrap();
    fs::write(path.join("233.txt"), vec![42; 233]).unwrap();
    fs::write(path.join("445.txt"), vec![42; 445]).unwrap();
    fs::write(path.join("heck.txt"), vec![0; 0]).unwrap();
    fs::create_dir_all(path.join("src/template")).unwrap();
    fs::write(path.join("src/template/hello.java"), vec![42; 514]).unwrap();
}

fn full_entry(path: &Path) -> DiskEntry {
    DiskEntry {
        name: p2b(path).to_vec(),
        metadata: Some(Metadata {
            file_type: FileType::Dir,
            ..Default::default()
        }),
        entries: vec![
            DiskEntry {
                name: b"afolder".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::Dir,
                    ..Default::default()
                }),
                entries: vec![
                    DiskEntry {
                        name: b"foo".to_vec(),
                        metadata: Some(Metadata {
                            file_type: FileType::File,
                            len: 666,
                            ..Default::default()
                        }),
                        entries: Vec::new(),
                    },
                    DiskEntry {
                        name: b"bar".to_vec(),
                        metadata: Some(Metadata {
                            file_type: FileType::File,
                            len: 89,
                            ..Default::default()
                        }),
                        entries: Vec::new(),
                    },
                ],
            },
            DiskEntry {
                name: b"bfolder".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::Dir,
                    ..Default::default()
                }),
                entries: vec![
                    DiskEntry {
                        name: b"cfolder".to_vec(),
                        metadata: Some(Metadata {
                            file_type: FileType::Dir,
                            ..Default::default()
                        }),
                        entries: vec![DiskEntry {
                            name: b"another".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 0,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        }],
                    },
                    DiskEntry {
                        name: b"foo".to_vec(),
                        metadata: Some(Metadata {
                            file_type: FileType::File,
                            len: 11,
                            ..Default::default()
                        }),
                        entries: Vec::new(),
                    },
                    DiskEntry {
                        name: b"bar".to_vec(),
                        metadata: Some(Metadata {
                            file_type: FileType::File,
                            len: 0,
                            ..Default::default()
                        }),
                        entries: Vec::new(),
                    },
                ],
            },
            DiskEntry {
                name: b"abc".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 233,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
            DiskEntry {
                name: "ldm".into(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 288,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
            DiskEntry {
                name: "vvv".into(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 12,
                    ..Default::default()
                }),
                entries: Vec::new(),
            },
        ],
    }
}

fn apply_full_entry(path: &Path) {
    fs::create_dir_all(path.join("afolder")).unwrap();
    fs::create_dir_all(path.join("bfolder")).unwrap();
    fs::create_dir_all(path.join("bfolder/cfolder")).unwrap();
    fs::write(path.join("abc"), vec![42; 233]).unwrap();
    fs::write(path.join("ldm"), vec![42; 288]).unwrap();
    fs::write(path.join("vvv"), vec![42; 12]).unwrap();
    fs::write(path.join("afolder/foo"), vec![42; 666]).unwrap();
    fs::write(path.join("afolder/bar"), vec![42; 89]).unwrap();
    fs::write(path.join("bfolder/foo"), vec![42; 11]).unwrap();
    File::create(path.join("bfolder/bar")).unwrap();
    File::create(path.join("bfolder/cfolder/another")).unwrap();
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            file_type: FileType::Unknown,
            len: 0,
            created: SystemTime::UNIX_EPOCH,
            accessed: SystemTime::UNIX_EPOCH,
            modified: SystemTime::UNIX_EPOCH,
            permissions_read_only: false,
        }
    }
}

#[test]
fn entry_from_empty_folder() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    let entry = DiskEntry::from_fs(path);
    compare_test_entry(
        DiskEntry {
            name: p2b(path).to_vec(),
            metadata: Some(Metadata {
                file_type: FileType::Dir,
                ..Default::default()
            }),
            entries: Vec::new(),
        },
        entry,
    )
}

#[test]
fn entry_from_single_file() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    let path = path.join("emm.txt");
    fs::write(&path, vec![42; 1000]).unwrap();
    let entry = DiskEntry::from_fs(&path);
    compare_test_entry(
        entry,
        DiskEntry {
            name: p2b(&path).to_vec(),
            metadata: Some(Metadata {
                file_type: FileType::File,
                len: 1000,
                ..Default::default()
            }),
            entries: Vec::new(),
        },
    );
}

#[test]
fn test_complex_entry_scanner() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    apply_complex_entry(path);
    let entry = DiskEntry::from_fs(path);
    compare_test_entry(entry, complex_entry(path));
}

#[test]
fn entry_from_full_folder() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    apply_full_entry(path);
    let entry = DiskEntry::from_fs(path);
    compare_test_entry(entry, full_entry(path));
}

#[cfg(target_family = "unix")]
mod symlink_tests {
    use super::*;

    fn create_complex_directory_with_symlink(path: &Path) {
        fs::create_dir(path.join("afolder")).unwrap();
        fs::create_dir(path.join("bfolder")).unwrap();
        fs::create_dir(path.join("bfolder/cfolder")).unwrap();
        unixfs::symlink(path.join("bfolder/cfolder"), path.join("dfolder")).unwrap();
        File::create(path.join("abc")).unwrap();
        File::create(path.join("ldm")).unwrap();
        File::create(path.join("vvv")).unwrap();
        fs::write(path.join("afolder/foo"), vec![42; 71]).unwrap();
        fs::write(path.join("afolder/kksk"), vec![42; 121]).unwrap();
        File::create(path.join("afolder/bar")).unwrap();
        File::create(path.join("bfolder/foo")).unwrap();
        File::create(path.join("bfolder/bar")).unwrap();
        fs::write(path.join("bfolder/kksk"), vec![42; 188]).unwrap();
        File::create(path.join("bfolder/cfolder/another")).unwrap();
        unixfs::symlink(path.join("afolder/bar"), path.join("afolder/baz")).unwrap();
        unixfs::symlink(path.join("afolder/foo"), path.join("bfolder/foz")).unwrap();
    }

    fn complex_entry_with_symlink(path: &Path) -> DiskEntry {
        DiskEntry {
            name: p2b(path).to_vec(),
            metadata: Some(Metadata {
                file_type: FileType::Dir,
                ..Default::default()
            }),
            entries: vec![
                DiskEntry {
                    name: b"afolder".to_vec(),
                    metadata: Some(Metadata {
                        file_type: FileType::Dir,
                        ..Default::default()
                    }),
                    entries: vec![
                        DiskEntry {
                            name: b"foo".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 71,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"bar".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 0,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"kksk".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 121,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"baz".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::Symlink,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                    ],
                },
                DiskEntry {
                    name: b"dfolder".to_vec(),
                    metadata: Some(Metadata {
                        file_type: FileType::Symlink,
                        ..Default::default()
                    }),
                    entries: Vec::new(),
                },
                DiskEntry {
                    name: b"bfolder".to_vec(),
                    metadata: Some(Metadata {
                        file_type: FileType::Dir,
                        ..Default::default()
                    }),
                    entries: vec![
                        DiskEntry {
                            name: b"cfolder".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::Dir,
                                ..Default::default()
                            }),
                            entries: vec![DiskEntry {
                                name: b"another".to_vec(),
                                metadata: Some(Metadata {
                                    file_type: FileType::File,
                                    len: 0,
                                    ..Default::default()
                                }),
                                entries: Vec::new(),
                            }],
                        },
                        DiskEntry {
                            name: b"foo".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 0,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"foz".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::Symlink,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"bar".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 0,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                        DiskEntry {
                            name: b"kksk".to_vec(),
                            metadata: Some(Metadata {
                                file_type: FileType::File,
                                len: 188,
                                ..Default::default()
                            }),
                            entries: Vec::new(),
                        },
                    ],
                },
                DiskEntry {
                    name: b"abc".to_vec(),
                    metadata: Some(Metadata {
                        file_type: FileType::File,
                        len: 0,
                        ..Default::default()
                    }),
                    entries: Vec::new(),
                },
                DiskEntry {
                    name: "ldm".into(),
                    metadata: Some(Metadata {
                        file_type: FileType::File,
                        len: 0,
                        ..Default::default()
                    }),
                    entries: Vec::new(),
                },
                DiskEntry {
                    name: "vvv".into(),
                    metadata: Some(Metadata {
                        file_type: FileType::File,
                        len: 0,
                        ..Default::default()
                    }),
                    entries: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn test_symlink() {
        let tempdir = TempDir::new().unwrap();
        let path = tempdir.path();
        create_complex_directory_with_symlink(path);
        let entry = DiskEntry::from_fs(path);
        compare_test_entry(entry, complex_entry_with_symlink(path));
    }
}

#[test]
fn test_simple_entry_merging() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    // DiskEntry::new()
}

#[test]
fn test_complex_entry_merging() {
    // Delete
    {
        let mut entry = complex_entry("/");
        entry.merge(&FsEvent {
            path: "/445.txt".into(),
            flag: EventFlag::Delete,
            id: 0,
        });
        let mut expected = complex_entry("/");
        let index = expected
            .entries
            .iter()
            .position(|x| x.name == b"445.txt")
            .unwrap();
        expected.entries.remove(index);
        compare_test_entry(entry, expected)
    }

    // Create uncreated file.
    {
        let mut entry = complex_entry("/");
        entry.merge(&FsEvent {
            path: "/asdfasdfknasdf.txt".into(),
            flag: EventFlag::Create,
            id: 0,
        });
        let expected = complex_entry("/");
        compare_test_entry(entry, expected)
    }

    // Modify uncreated file.
    {
        let mut entry = complex_entry("/");
        entry.merge(&FsEvent {
            path: "/11451419190810.txt".into(),
            flag: EventFlag::Modify,
            id: 0,
        });
        let expected = complex_entry("/");
        compare_test_entry(entry, expected)
    }
}

#[test]
fn test_on_disk_entry_modifying() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    apply_complex_entry(path);

    // Write 6 extra bytes to 445.txt.
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.join("445.txt"))
            .unwrap();
        file.write_all(b"hello?").unwrap();
        drop(file);
    }
    // Write 8 extra bytes to hello.java.
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.join("src/template/hello.java"))
            .unwrap();
        file.write_all(b"asdfasdf").unwrap();
        drop(file);
    }

    let mut entry = DiskEntry::from_fs(path);
    entry.merge(&FsEvent {
        path: path.join("445.txt"),
        flag: EventFlag::Modify,
        id: 0,
    });
    entry.merge(&FsEvent {
        path: path.join("src/template/hello.java"),
        flag: EventFlag::Modify,
        id: 0,
    });
    let x = entry.entries.iter().find(|x| x.name == b"445.txt").unwrap();
    let metadata = x.metadata.as_ref().unwrap();
    assert_eq!(metadata.permissions_read_only, false);
    assert_ne!(metadata.created, SystemTime::UNIX_EPOCH);
    assert_ne!(metadata.modified, SystemTime::UNIX_EPOCH);
    assert_ne!(metadata.accessed, SystemTime::UNIX_EPOCH);
    assert_eq!(metadata.len, 451);
    assert_eq!(metadata.file_type, FileType::File);

    let src = entry.entries.iter().find(|x| x.name == b"src").unwrap();
    let template = src.entries.iter().find(|x| x.name == b"template").unwrap();
    let x = template
        .entries
        .iter()
        .find(|x| x.name == b"hello.java")
        .unwrap();
    let metadata = x.metadata.as_ref().unwrap();
    assert_eq!(metadata.permissions_read_only, false);
    assert_ne!(metadata.created, SystemTime::UNIX_EPOCH);
    assert_ne!(metadata.modified, SystemTime::UNIX_EPOCH);
    assert_ne!(metadata.accessed, SystemTime::UNIX_EPOCH);
    assert_eq!(metadata.len, 522);
    assert_eq!(metadata.file_type, FileType::File);
}

#[test]
fn test_on_disk_entry_deleting() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    apply_complex_entry(path);

    // Remove `445.txt`.
    fs::remove_file(path.join("445.txt")).unwrap();
    // Remove `template` folder.
    fs::remove_dir_all(path.join("src/template")).unwrap();

    let mut entry = DiskEntry::from_fs(path);
    entry.merge(&FsEvent {
        path: path.join("445.txt"),
        flag: EventFlag::Delete,
        id: 0,
    });
    entry.merge(&FsEvent {
        path: path.join("src/template"),
        flag: EventFlag::Delete,
        id: 0,
    });

    let mut expected = complex_entry(path);
    let x = expected
        .entries
        .iter()
        .position(|x| x.name == b"445.txt")
        .unwrap();
    expected.entries.remove(x);

    let src = expected
        .entries
        .iter_mut()
        .find(|x| x.name == b"src")
        .unwrap();
    let template = src
        .entries
        .iter()
        .position(|x| x.name == b"template")
        .unwrap();
    src.entries.remove(template);

    compare_test_entry(entry, expected);
}

#[test]
fn test_on_disk_entry_creating() {
    let tempdir = TempDir::new().unwrap();
    let path = tempdir.path();
    apply_complex_entry(path);

    // Create `foobar.txt`.
    fs::write(path.join("foobar.txt"), b"donoughliu123").unwrap();
    // Create `fook/barm/tmp`.
    fs::create_dir_all(path.join("fook/barm/")).unwrap();
    fs::write(path.join("fook/barm/tmp"), b"1234567890").unwrap();

    let mut entry = DiskEntry::from_fs(path);
    entry.merge(&FsEvent {
        path: path.join("foobar.txt"),
        flag: EventFlag::Create,
        id: 0,
    });
    entry.merge(&FsEvent {
        path: path.join("fook/barm/tmp"),
        flag: EventFlag::Create,
        id: 0,
    });

    let mut expected = complex_entry(path);
    expected.entries.push(DiskEntry::new(
        b"foobar.txt",
        Some(Metadata {
            file_type: FileType::File,
            len: 13,
            ..Default::default()
        }),
    ));
    let tmp_entry = DiskEntry {
        name: b"fook".to_vec(),
        metadata: Some(Metadata {
            file_type: FileType::Dir,
            ..Default::default()
        }),
        entries: vec![DiskEntry {
            name: b"barm".to_vec(),
            metadata: Some(Metadata {
                file_type: FileType::Dir,
                ..Default::default()
            }),
            entries: vec![DiskEntry {
                name: b"tmp".to_vec(),
                metadata: Some(Metadata {
                    file_type: FileType::File,
                    len: 10,
                    ..Default::default()
                }),
                entries: vec![],
            }],
        }],
    };
    expected.entries.push(tmp_entry);

    compare_test_entry(entry, expected);
}
