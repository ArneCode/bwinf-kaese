use rustc_hash::{FxHashMap, FxHasher};
use std::{
    env::args,
    fs,
    hash::{BuildHasherDefault, Hash},
    rc::Rc,
    time::Instant,
};
use uuid::Uuid;
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Piece(u32, u32);
impl TryFrom<Vec<&str>> for Piece {
    type Error = std::num::ParseIntError;
    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        assert!(value.len() == 2);
        let width = value[0].parse()?;
        let height = value[1].parse()?;
        if width > height {
            Ok(Self(width, height))
        } else {
            Ok(Self(height, width))
        }
    }
}
#[derive(Debug, Clone)]
struct PiecesMap {
    base: Rc<FxHashMap<Piece, u32>>,
    base_id: Uuid,
    added: FxHashMap<Piece, u32>,
}

impl PiecesMap {
    fn new(map: FxHashMap<Piece, u32>) -> Self {
        let base_id = Uuid::new_v4();
        Self {
            base: Rc::new(map),
            base_id,
            added: FxHashMap::default(),
        }
    }
    fn get(&self, k: &Piece) -> Option<&u32> {
        if let Some(result) = self.added.get(k) {
            Some(result)
        } else if let Some(result) = self.base.get(k) {
            Some(result)
        } else {
            None
        }
    }

    fn insert(&mut self, k: Piece, v: u32) -> Option<u32> {
        self.added.insert(k, v)
    }

    fn make_copy(&mut self) -> Self {
        if self.added.len() < self.base.len() / 40 {
            Self {
                base: self.base.clone(),
                base_id: self.base_id,
                added: self.added.clone(),
            }
        } else {
            println!("merging");
            let mut base = FxHashMap::with_capacity_and_hasher(
                self.base.len(),
                BuildHasherDefault::<FxHasher>::default(),
            );
            for (k, v) in self.base.as_ref() {
                let v = if let Some(v) = self.added.get(k) {
                    v.clone()
                } else {
                    v.clone()
                };
                if v == 0 {
                    continue;
                }
                base.insert(k.clone(), v);
            }
            self.base = Rc::new(base);
            self.added = FxHashMap::default();
            self.clone()
        }
    }
}
#[derive(Debug)]
struct Path {
    prev: Option<Rc<Path>>,
    value: Piece,
    is_added: bool,
}

impl Path {
    fn new(value: Piece) -> Self {
        Self {
            prev: None,
            value,
            is_added: false,
        }
    }
    fn make_next(self: &Rc<Self>, value: Piece, is_added: bool) -> Rc<Self> {
        Rc::new(Self {
            prev: Some(self.clone()),
            value,
            is_added,
        })
    }
    fn to_array(self: &Rc<Path>) -> Vec<Rc<Path>> {
        let mut nodes = vec![];
        let mut node = self.clone();
        nodes.push(node.clone());
        while let Some(prev) = &node.prev {
            node = prev.clone();
            nodes.push(node.clone());
        }
        nodes
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Cheese {
    //a<=b<=c
    size: [u32; 3],
}
impl Cheese {
    fn new(size: [u32; 3]) -> Self {
        Self { size }
    }

    pub fn get_sides(&self) -> Vec<Piece> {
        vec![
            Piece(self.size[1], self.size[2]),
            Piece(self.size[0], self.size[2]),
            Piece(self.size[0], self.size[1]),
        ]
    }
    fn expand_side(&self, n: usize) -> Cheese {
        let mut size = self.size;
        size[n] += 1;
        size.sort_unstable_by_key(|size| std::cmp::Reverse(*size));
        // println!("size: {:?}", size);
        Cheese { size }
    }
    pub fn gen_poss_paths(
        &self,
        path: Rc<Path>,
        pieces: Box<PiecesMap>,
    ) -> Vec<(Cheese, Rc<Path>, Box<PiecesMap>)> {
        let paths = self
            .get_sides()
            .into_iter()
            .enumerate()
            .filter_map(|(i, side)| {
                let n_pieces = pieces.get(&side)?;
                if n_pieces == &0 {
                    return None;
                }
                Some((i, side, n_pieces - 1))
            })
            .collect::<Vec<_>>();
        let paths = if !paths.is_empty() {
            let pieces_ptr = Box::into_raw(pieces);
            paths
                .iter()
                .enumerate()
                .rev()
                .map(move |(i, (side_n, side, n_pieces))| {
                    let mut pieces = if i == 0 {
                        //if this is the last copy that will be used of the pieces, use as is
                        unsafe { Box::from_raw(pieces_ptr) }
                    } else {
                        //else copy
                        Box::new(unsafe { pieces_ptr.as_mut() }.unwrap().make_copy())
                    };
                    pieces.insert(side.clone(), *n_pieces);
                    let new_path = path.make_next(side.clone(), false);
                    let new_cheese = self.expand_side(*side_n);
                    (new_cheese, new_path, pieces)
                })
                .collect()
        } else {
            vec![]
        };
        paths
    }
}
impl From<Piece> for Cheese {
    fn from(value: Piece) -> Self {
        Cheese::new([value.0, value.1, 1])
    }
}
fn gen_pieces_map(pieces: Vec<Piece>) -> PiecesMap {
    let mut pieces_map: FxHashMap<Piece, u32> = FxHashMap::default();
    let mut max_n = 0;
    let mut n_multiple = 0;
    for piece in pieces {
        if let Some(n) = pieces_map.get_mut(&piece) {
            *n += 1;
            n_multiple += 1;
            if n > &mut max_n {
                max_n = *n;
            }
        } else {
            pieces_map.insert(piece, 1);
        }
    }
    println!(
        "max number of one piece: {}, n_multiple: {}, number of different pieces: {}",
        max_n,
        n_multiple,
        pieces_map.len()
    );
    PiecesMap::new(pieces_map)
}
fn main() {
    let args: Vec<String> = args().collect();
    let s = fs::read_to_string(&args[1]).expect("couldn't read file");
    let mut lines = s.split("\r\n");
    let n_pieces: usize = lines
        .next()
        .expect("empty data file")
        .parse()
        .expect("couldn't extract number of pieces");
    let pieces: Vec<Piece> = lines
        .filter_map(|line| -> Option<Piece> {
            if line.is_empty() {
                None
            } else {
                Some(
                    line.split(' ')
                        .collect::<Vec<&str>>()
                        .try_into()
                        .expect("couldn't parse line"),
                )
            }
        })
        .collect();
    assert_eq!(n_pieces, pieces.len());
    println!("read {} pieces from the file", n_pieces);
    let start = Instant::now();
    let pieces_map = Box::new(gen_pieces_map(pieces));
    //find possible start pieces_map
    let mut poss_paths = vec![];
    for piece in pieces_map.base.keys() {
        let cheese = Cheese::new([piece.0, piece.1, 0]);
        // println!("new possible cheese: {:?}", cheese);
        // poss_paths.extend(cheese.gen_poss_paths(pieces_map, &vec![]));
        poss_paths.push((
            cheese,
            Rc::new(Path::new(piece.clone())),
            pieces_map.clone(),
        ));
    }
    let mut n_checks = 0;
    let mut i = 0;
    while i < n_pieces {
        let paths_len = poss_paths.len();
        if i % 10000 == 0 || poss_paths.len() > 1 {
            println!("a, {} {}/{}", poss_paths.len(), i, n_pieces);
        }
        let mut new_paths = vec![];
        for (cheese, path, pieces) in poss_paths.into_iter() {
            if paths_len < 3 {
                // println!("{:?}", path);
            }
            let paths = cheese.gen_poss_paths(path, pieces);
            new_paths.extend(paths);
        }
        poss_paths = new_paths;
        let mut other_cheeses: FxHashMap<&Cheese, &Box<PiecesMap>> = FxHashMap::default();
        let paths_filter = poss_paths
            .iter()
            .map(|(cheese, _, pieces)| {
                if let Some(other_pieces) = other_cheeses.get(&cheese) {
                    let keys = if pieces.base_id == other_pieces.base_id {
                        pieces.added.keys()
                    } else {
                        n_checks += 1;
                        pieces.base.keys()
                    };
                    for piece in keys {
                        if pieces.get(piece) != other_pieces.get(piece) {
                            return true;
                        }
                    }
                    false
                    //other_pieces.added != pieces.added && other_pieces.base != pieces.base
                } else {
                    other_cheeses.insert(cheese, pieces);
                    true
                }
            })
            .collect::<Vec<_>>();
        poss_paths = poss_paths
            .into_iter()
            .zip(paths_filter)
            .filter(|(_, v)| *v)
            .map(|(v, _)| v)
            .collect();
        i += 1;
    }
    if poss_paths.is_empty() {
        println!("found no solution");
    }
    println!("len: {}", poss_paths.len());
    for (cheese, path, _) in poss_paths {
        println!("found solution: {:#?}", cheese);
        let path = path.to_array().into_iter().rev().collect::<Vec<_>>();
        println!("first piece: {:?}", path[0].value);
    }
    println!("test");
    let elapsed = start.elapsed();
    println!("took: {:?}", elapsed);
    //println!("Hello, world! pieces: {:#?}", pieces);
}
