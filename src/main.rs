use rand::{prelude::*, rngs::ThreadRng, seq::SliceRandom, thread_rng};
use rustc_hash::{FxHashMap, FxHasher};
use std::{
    env::args,
    fs,
    hash::{BuildHasherDefault, Hash},
    mem,
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
            // panic!();
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
    fn merge_hashmaps(&self) -> FxHashMap<Piece, u32> {
        let mut new_map = FxHashMap::with_capacity_and_hasher(
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
            new_map.insert(k.clone(), v);
        }
        new_map
    }
    fn merge(&mut self) {
        // println!("merging");
        let base = self.merge_hashmaps();
        self.base = Rc::new(base);
        self.added = FxHashMap::default();
    }
    fn make_copy(&mut self) -> Self {
        if self.added.len() > self.base.len() / 10 {
            self.merge();
        }
        self.clone()
    }
    fn clone_only(&self, other_map: Box<PiecesMap>) -> Self {
        let mut new_map = FxHashMap::default();
        for piece in other_map.base.keys() {
            let amount = match self.get(piece) {
                Some(0) => continue,
                Some(amt) => amt,
                None => continue,
            };
            new_map.insert(piece.clone(), *amount);
        }
        Self::new(new_map)
    }
    fn clone_without(&self, removed: &Vec<Piece>) -> Self {
        if removed.is_empty() {
            return self.clone();
        }
        let mut new_map = self.merge_hashmaps();
        for piece in removed {
            let mut do_remove = false;
            if let Some(v) = new_map.get_mut(piece) {
                *v -= 1;
                if *v == 0 {
                    do_remove = true;
                }
            }
            if do_remove {
                new_map.remove(piece);
            }
        }
        Self::new(new_map)
    }
}
#[derive(Clone)]
struct Path {
    curr: Rc<HistPoint>,
    start_piece: Piece,
    len: usize,
    n_added: u32,
}
impl Path {
    fn new(value: Piece) -> Self {
        Self {
            curr: Rc::new(HistPoint::new(value.clone())),
            start_piece: value,
            len: 1,
            n_added: 0,
        }
    }
    fn extend_real(&self, value: Piece) -> Self {
        Self {
            curr: self.curr.make_next(value, false),
            start_piece: self.start_piece.clone(),
            len: self.len + 1,
            n_added: self.n_added,
        }
    }
    fn extend_added(&self, value: Piece) -> Self {
        Self {
            curr: self.curr.make_next(value, true),
            start_piece: self.start_piece.clone(),
            len: self.len,
            n_added: self.n_added + 1,
        }
    }
}
#[derive(Debug, Clone)]
struct HistPoint {
    prev: Option<Rc<HistPoint>>,
    value: Piece,
    is_added: bool,
}

impl HistPoint {
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
    fn to_array(self: &Rc<HistPoint>) -> Vec<Rc<HistPoint>> {
        let mut nodes = vec![];
        let mut node = self.clone();
        nodes.push(node.clone());
        while let Some(prev) = &node.prev {
            node = prev.clone();
            nodes.push(node.clone());
        }
        nodes.pop();
        nodes
    }
    fn get_real_pieces(self: &Rc<HistPoint>) -> Vec<Piece> {
        self.to_array()
            .iter()
            .filter(|pt| !pt.is_added)
            .map(|pt| pt.value.clone())
            .collect()
    }
    fn get_pieces(self: &Rc<HistPoint>) -> Vec<Piece> {
        self.to_array().iter().map(|pt| pt.value.clone()).collect()
    }
}
impl Drop for Path {
    fn drop(&mut self) {
        let curr = if let Some(curr) = Rc::get_mut(&mut self.curr) {
            curr
        } else {
            return;
        };
        let mut maybe_node = mem::take(&mut curr.prev);
        while let Some(mut node) = maybe_node {
            if let Some(node_mut) = Rc::get_mut(&mut node) {
                maybe_node = mem::take(&mut node_mut.prev);
            } else {
                return;
            }
        }
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
    //get the side lengths used to compose a side
    pub fn get_sides_n() -> Vec<(usize, usize)> {
        vec![(1, 2), (0, 2), (0, 1)]
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

    fn find_missing(
        &self,
        updated_sides: Vec<bool>,
        pieces: &Box<PiecesMap>,
    ) -> Vec<(usize, Piece, Option<u32>)> {
        Cheese::get_sides_n()
            .into_iter()
            .enumerate()
            .flat_map(|(i, (other_a, other_b))| {
                if updated_sides[other_a] && updated_sides[other_b] {
                    //if the other sides have already been added to, they can't have a missing piece
                    return vec![];
                }
                let len_x = self.size[other_a];
                let len_y = self.size[other_b];
                let len_z = self.size[i];
                let mut poss_missing = vec![];
                if !updated_sides[other_a] {
                    poss_missing.push((other_a, Piece(len_y, len_z), Piece(len_x + 1, len_y)))
                }
                if !updated_sides[other_b] {
                    poss_missing.push((other_b, Piece(len_x, len_z), Piece(len_x, len_y + 1)))
                }
                poss_missing
            })
            .filter_map(|(added_i, added, new)| {
                //look up if a matching piece exists
                let n_pieces = pieces.get(&new)?;
                if n_pieces == &0 {
                    return None;
                }
                // println!("found piece: {:?}, n_pieces: {}", new, n_pieces);
                // Some((i, new, n_pieces - 1, Some(added)))
                // todo!()
                Some((added_i, added, None))
            })
            .collect::<Vec<_>>()
    }
    pub fn gen_poss_paths(
        &self,
        path: Path,
        pieces: Box<PiecesMap>,
    ) -> Vec<(Cheese, Path, Box<PiecesMap>)> {
        //<<<<<<< Updated upstream
        // let mut sides_added = vec![];
        let mut sides_seen = vec![];
        let mut updated_sides = vec![false; 3];
        let mut paths = self
            .get_sides()
            .into_iter()
            .enumerate()
            .filter(|(_, side)| {
                //check if two sides of the cheese are equal
                if sides_seen.iter().any(|other| other == side) {
                    false
                } else {
                    sides_seen.push(side.clone());
                    true
                }
            })
            .filter_map(|(i, side)| {
                let n_pieces = pieces.get(&side)?;
                if n_pieces == &0 {
                    return None;
                }
                updated_sides[i] = true;
                Some((i, side, Some(n_pieces - 1)))
            })
            .collect::<Vec<_>>();
        // /*
        // check for possible missing pieces
        if paths.is_empty() {
            // println!("paths is empty");
            paths = self.find_missing(updated_sides, &pieces); //*/
        }
        let paths = if !paths.is_empty() {
            let pieces_ptr = Box::into_raw(pieces);
            paths
                .into_iter()
                .enumerate()
                .rev()
                .map(move |(i, (side_n, side, new_n_pieces))| {
                    let mut pieces = if i == 0 {
                        //if this is the last copy that will be used of the pieces, use as is
                        unsafe { Box::from_raw(pieces_ptr) }
                    } else {
                        //else copy
                        Box::new(unsafe { pieces_ptr.as_mut() }.unwrap().make_copy())
                    };
                    let new_path = if let Some(n_pieces) = new_n_pieces {
                        //piece is real
                        pieces.insert(side.clone(), n_pieces);
                        path.extend_real(side.clone())
                        // path.clone()
                    } else {
                        //piece is added
                        path.extend_added(side.clone())
                    };
                    // let new_path = new_path.extend(side.clone(), false);

                    let new_cheese = self.expand_side(side_n);
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
fn gen_pieces_map(pieces: &Vec<Piece>) -> PiecesMap {
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
            pieces_map.insert(piece.clone(), 1);
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
fn load_pieces(path: &str) -> Vec<Piece> {
    let s = fs::read_to_string(path).expect("couldn't read file");
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
    pieces
}
type PossPath = (Cheese, Path, Box<PiecesMap>);
fn gen_path_starts(pieces_map: &Box<PiecesMap>) -> Vec<PossPath> {
    //find possible start pieces_map
    let mut poss_paths: Vec<(Cheese, Path, Box<PiecesMap>)> = vec![];
    for piece in pieces_map.base.keys() {
        let cheese = Cheese::new([piece.0, piece.1, 0]);
        // println!("new possible cheese: {:?}", cheese);
        // poss_paths.extend(cheese.gen_poss_paths(pieces_map, &vec![]));
        poss_paths.push((cheese, Path::new(piece.clone()), pieces_map.clone()));
    }
    poss_paths
}
fn filter_multiple_paths(mut poss_paths: Vec<PossPath>) -> Vec<PossPath> {
    let mut other_cheeses: FxHashMap<&Cheese, (&Box<PiecesMap>, Path, usize)> =
        FxHashMap::default();
    let mut used_paths: Vec<usize> = vec![];
    for (i, (cheese, path, pieces)) in poss_paths.iter().enumerate() {
        let mut add_to_others = true;
        if let Some((other_pieces, other_path, idx)) = other_cheeses.get(&cheese) {
            add_to_others = false;
            let keys = if pieces.base_id == other_pieces.base_id {
                pieces.added.keys()
            } else {
                //FIXME
                pieces.base.keys()
            };
            for piece in keys {
                if pieces.get(piece) != other_pieces.get(piece) {
                    add_to_others = true;
                    break;
                }
            }
            if !add_to_others {
                if other_path.n_added > path.n_added {
                    used_paths[*idx] = i;
                }
                // println!("merge");
            }
            // let arr = path.curr.get_pieces().into_iter().rev().collect::<Vec<_>>();
            // let other_arr = other_path
            //     .curr
            //     .get_pieces()
            //     .into_iter()
            //     .rev()
            //     .collect::<Vec<_>>();
            // println!("cheese: {:?}", cheese);
            // println!("merged {}:\n\t{:?}", arr.len(), arr);
            // println!("other:\n\t{:?}", other_arr);
            //other_pieces.added != pieces.added && other_pieces.base != pieces.base
        }
        if add_to_others {
            other_cheeses.insert(&cheese, (&pieces, path.clone(), used_paths.len()));
            used_paths.push(i);
        }
    }
    used_paths.sort();
    used_paths
        .into_iter()
        .rev()
        .map(|idx| poss_paths.swap_remove(idx))
        .collect()
}
fn cheese_ok(cheese: Cheese, min_path_len: usize) -> bool {
    !cheese.size.iter().any(|size| *size == min_path_len as u32)
}
fn construct_cheese(start: PossPath, min_path_len: usize) -> Option<(Cheese, Path)> {
    let mut poss_paths = vec![start];
    let mut i = 0;
    loop {
        if i % 10000 == 0 && i != 0 || (poss_paths.len() > 5 && i % 100 == 0) {
            println!("a, {} {}", poss_paths.len(), i);
        }
        let mut new_paths = vec![];
        let curr_result = {
            let (cheese, path, _) = &poss_paths[0];
            (cheese.clone(), path.clone())
        };
        for (cheese, path, pieces) in poss_paths.into_iter() {
            let paths = cheese.gen_poss_paths(path, pieces);
            new_paths.extend(paths);
        }
        if new_paths.is_empty() {
            return if i >= min_path_len && cheese_ok(curr_result.0, min_path_len) {
                Some(curr_result)
            } else {
                None
            };
        }
        poss_paths = filter_multiple_paths(new_paths);
        i += 1;
    }
}
fn filter_top_paths(paths: Vec<Vec<PossPath>>) -> Vec<Vec<PossPath>> {
    println!("n_paths: {}", paths.len());

    let start_piece = &paths[1][0].1.start_piece;
    println!("first piece: {:?}", start_piece);
    let n_added = &paths[1][0].1.n_added;
    println!("n_added: {}", n_added);
    // println!("last piece: {:?}", path.last().unwrap());
    todo!()
}
// fn advance_paths(poss_paths: Vec<(Cheese, Path, Box<PiecesMap>)>) -> PossPaths {}
fn construct_cheeses(pieces: Box<PiecesMap>, n_pieces: usize) -> Vec<(Cheese, Path)> {
    // let poss_paths = {
    let mut pieces_map = pieces;
    //find possible start pieces_map
    let mut results = vec![];
    let mut used_pieces = vec![];
    let mut min_path_len = n_pieces * 3 / 4;
    'main_loop: while min_path_len > 2 {
        let mut found_cheese = false;
        let keys = pieces_map.base.keys().cloned().collect::<Vec<Piece>>();
        let mut top_paths = keys
            .iter()
            .map(|piece| {
                let cheese = Cheese::new([piece.0, piece.1, 0]);
                let start = (cheese, Path::new(piece.clone()), pieces_map.clone());
                vec![start]
            })
            .collect::<Vec<_>>();
        let mut i = 0;
        'top_loop: while !top_paths.is_empty() {
            if i % 10000 == 0 {
                println!("n_paths (top): {}, i: {i}", top_paths.len());
            }
            let mut new_top_paths = vec![];
            let n_top = top_paths.len();
            'sub_loop: for sub_paths in top_paths {
                if i % 10000 == 0 && i != 0 || sub_paths.len() > 4 {
                    println!("n_paths (sub): {}", sub_paths.len());
                }
                // println!("new possible cheese: {:?}", cheese);
                // poss_paths.extend(cheese.gen_poss_paths(pieces_map, &vec![]));
                // let result = {
                // let start = poss_paths;
                // let mut poss_paths = vec![start];
                // let mut i = -1;
                // if i % 10000 == 0 && i != 0 || (poss_paths.len() > 5 && i % 100 == 0) {
                //     println!("a, {} {}", poss_paths.len(), i);
                // }
                let mut new_paths = vec![];
                let curr_result = {
                    let (cheese, path, _) = &sub_paths[0];
                    (cheese.clone(), path.clone())
                };
                for (cheese, path, pieces) in sub_paths.into_iter() {
                    let paths = cheese.gen_poss_paths(path, pieces);
                    new_paths.extend(paths);
                }
                // if new_paths.is_empty() {
                //     return if i >= min_path_len && cheese_ok(curr_result.0, min_path_len) {
                //         Some(curr_result)
                //     } else {
                //         None
                //     };
                // }
                if i % 10 == 0 {
                    //remove the paths with more added pieces
                    let min_added = new_paths
                        .iter()
                        .fold(u32::MAX, |acc, (_, path, _)| acc.min(path.n_added));
                    new_paths = new_paths
                        .into_iter()
                        .filter(|(_, path, _)| path.n_added <= min_added)
                        .collect();
                }
                if new_paths.is_empty() {
                    if i >= min_path_len && cheese_ok(curr_result.0, min_path_len) {
                        let (cheese, path) = curr_result;
                        let new_used_pieces = path.curr.get_real_pieces();
                        println!(
                            "found cheese: {:?}, len: {}, n_added: {}, n_top: {}",
                            cheese, path.len, path.n_added, n_top
                        );
                        // panic!();
                        pieces_map = Box::new(pieces_map.clone_without(&new_used_pieces));
                        used_pieces.extend(new_used_pieces);
                        println!("total len: {}/{}", path.len, n_pieces);
                        min_path_len = (n_pieces - used_pieces.len()) * 3 / 4;
                        results.push((cheese, path));
                        // found_cheese = true;
                        continue 'main_loop;
                    }
                } else {
                    new_top_paths.push(filter_multiple_paths(new_paths));
                }
                // i += 1;
                // };
                // if let Some((cheese, path)) = result {
                //     let new_used_pieces = path.curr.get_real_pieces();
                //     println!(
                //         "found cheese: {:?}, len: {}, n_added: {}",
                //         cheese, path.len, path.n_added
                //     );
                //     // panic!();
                //     pieces_map = Box::new(pieces_map.clone_without(&new_used_pieces));
                //     used_pieces.extend(new_used_pieces);
                //     println!("total len: {}/{}", path.len, n_pieces);
                //     min_path_len = (n_pieces - used_pieces.len()) * 3 / 4;
                //     results.push((cheese, path));
                //     found_cheese = true;
                // }
            }
            if i == 100 {
                // new_top_paths = filter_top_paths(new_top_paths);
            }
            top_paths = new_top_paths;
            i += 1;
        }
        if !found_cheese {
            min_path_len /= 2;
        }
    }
    if used_pieces.len() < n_pieces {
        println!("not all pieces used!!, {:?}", pieces_map);
        println!("total len: {}/{}", used_pieces.len(), n_pieces);
        println!("pieces left: {}", pieces_map.base.values().sum::<u32>());
    } else if used_pieces.len() > n_pieces {
        println!("used: {:?}", used_pieces);
        panic!("error, used {} out of {}", used_pieces.len(), n_pieces);
    }

    // poss_paths
    // };
    // poss_paths
    //     .into_iter()
    //     .filter_map(|start| construct_cheese(start, min_path_len))
    //     .collect()
    results
}
fn eat_pieces(pieces: Vec<Piece>, rng: &mut ThreadRng, prob: f64) -> Vec<Piece> {
    let mut new_pieces = vec![];
    let mut last_eaten = false;
    let mut n_eaten = 0;
    for (i, piece) in pieces.into_iter().enumerate() {
        if rng.gen_bool(1.0 - prob) || last_eaten {
            new_pieces.push(piece);
            last_eaten = false;
        } else {
            last_eaten = true;
            n_eaten += 1;
        }
    }
    println!("ate {} pieces", n_eaten);
    new_pieces
}
fn main() {
    // let args: Vec<String> = args().collect();
    let mut pieces = args()
        .skip(1)
        .flat_map(|path| load_pieces(&path))
        .collect::<Vec<_>>();
    let mut rng = thread_rng();
    pieces.shuffle(&mut rng);
    println!("first pieces: {:?}", &pieces[0..pieces.len().min(10)]);
    let pieces = eat_pieces(pieces, &mut rng, 0.00001);
    // let n_pieces = pieces.len();
    // println!("pieces: {:?}", pieces);
    let start = Instant::now();
    let pieces_map = Box::new(gen_pieces_map(&pieces));
    let result = construct_cheeses(pieces_map, pieces.len());
    if result.is_empty() {
        println!("found no solution");
    }
    for (cheese, path) in &result {
        println!("found solution: {:#?}", cheese);
        let path = path.curr.get_pieces().into_iter().rev().collect::<Vec<_>>();
        println!("first piece: {:?}", path[0]);
        println!("last piece: {:?}", path.last().unwrap());
        // println!("path: {:?}", path);
    }
    let elapsed = start.elapsed();
    println!("took: {:?}", elapsed);
    // unsafe {
    //     DO_LOG = true;
    // }
    // println!("test");
    //println!("Hello, world! pieces: {:#?}", pieces);
}
