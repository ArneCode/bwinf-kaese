use std::{collections::HashMap, fs};
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Piece(u32, u32);
impl Piece {
    pub fn rotated(&self) -> Self {
        Piece(self.1, self.0)
    }
}
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
type PiecesMap = HashMap<Piece, u32>;
#[derive(Debug)]
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
        let mut size = self.size.clone();
        size[n] += 1;
        size.sort_unstable_by_key(|size| std::cmp::Reverse(*size));
        // println!("size: {:?}", size);
        Cheese { size }
    }
    pub fn gen_poss_paths(&self, pieces: &PiecesMap) -> Vec<(Cheese, PiecesMap)> {
        let mut paths = vec![];
        for (i, side) in self.get_sides().iter().enumerate() {
            if let Some(_) = pieces.get(&side) {
                let mut new_map = pieces.clone();
                let n = {
                    let n = new_map.get_mut(&side).unwrap();
                    *n -= 1;
                    *n
                };
                let new_cheese = self.expand_side(i);
                if n == 0 {
                    new_map.remove(&side);
                    if new_map.len() == 0 {
                        println!("found solution, cheese: {:#?}", new_cheese);
                    }
                }
                paths.push((new_cheese, new_map));
            }
        }
        paths
    }
}
impl From<Piece> for Cheese {
    fn from(value: Piece) -> Self {
        Cheese::new([value.0, value.1, 1])
    }
}
fn gen_pieces_map(pieces: Vec<Piece>) -> PiecesMap {
    let mut pieces_map: HashMap<Piece, u32> = HashMap::new();
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
    println!("read {} pieces from file\nmax number of one piece: {}, n_multiple: {}, number of different pieces: {}", pieces_map.len(), max_n, n_multiple, pieces_map.len());
    pieces_map
}
fn main() {
    let s = fs::read_to_string("data/kaese4.txt").expect("couldn't read file");
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
                    line.split(" ")
                        .collect::<Vec<&str>>()
                        .try_into()
                        .expect("couldn't parse line"),
                )
            }
        })
        .collect();
    assert_eq!(n_pieces, pieces.len());
    let pieces_map = gen_pieces_map(pieces);
    //find possible start pieces_map
    let mut poss_paths = vec![];
    for (piece, _) in pieces_map.iter() {
        let cheese = Cheese::new([piece.0, piece.1, 0]);
        println!("new possible cheese: {:?}", cheese);
        poss_paths.extend(cheese.gen_poss_paths(&pieces_map));
    }
    while !poss_paths.is_empty() {
        println!("a");
        let mut new_paths = vec![];
        for (cheese, pieces) in poss_paths.iter() {
            let paths = cheese.gen_poss_paths(&pieces);
            new_paths.extend(paths);
        }
        poss_paths = new_paths;
    }
    //println!("Hello, world! pieces: {:#?}", pieces);
}
