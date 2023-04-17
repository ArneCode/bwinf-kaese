use std::cmp::Ordering;

use crate::{
    cheese::{Cheese, Piece},
    pieces_map::PiecesMap,
    prev_pieces::PrevPieces,
};
//Ein möglicher Pfad
pub struct PossPath {
    cheese: Cheese, //Der mögliche Käse
    prev_pieces: PrevPieces,
    pieces_left: Box<PiecesMap>,
}

impl PossPath {
    ///erzeugt einen neuen möglichen Pfad
    pub fn new(cheese: Cheese, path: PrevPieces, pieces: Box<PiecesMap>) -> Self {
        Self {
            cheese,
            prev_pieces: path,
            pieces_left: pieces,
        }
    }
    ///erzeugt neue Pfade, indem es die neuen Seiten an den Käse anfügt
    fn gen_new_paths(self, find_missing: bool) -> Vec<PossPath> {
        self.cheese
            .gen_poss_paths(self.prev_pieces, self.pieces_left, find_missing)
    }
}
//Lässt aus den top-Paths nur den Pfad mit den wenigsten hinzugefügten Stücken über
fn filter_top_paths(paths: Vec<Vec<PossPath>>) -> Vec<Vec<PossPath>> {
    let min_added_path = paths
        .into_iter()
        .min_by_key(|path| {
            path.iter()
                .min_by_key(|poss_path| poss_path.prev_pieces.n_added)
                .unwrap()
                .prev_pieces
                .n_added
        })
        .unwrap();
    vec![min_added_path]
}
//lässt aus den sub-Paths nur den Pfad mit den wenigsten hinzugefügten Stücken übrig
//entfernt die Pfade mit mehr hinzugefügten Stücken
fn filter_sub_paths(mut paths: Vec<PossPath>) -> Vec<PossPath> {
    let min_added = paths.iter().fold(u32::MAX, |acc, poss_path| {
        acc.min(poss_path.prev_pieces.n_added)
    });
    paths.retain(|poss_path| poss_path.prev_pieces.n_added <= min_added);
    paths
}
// Setzt nur einen Käse zusammen, wird immer wieder von construct_cheeses aufgerufen
fn construct_cheese(
    mut top_paths: Vec<Vec<PossPath>>, // Die Pfade, nach Startstück getrennt
    min_path_len: usize,               // Die minimale Länge eines Pfades
    find_missing: bool,                // Ob nach fehlenden Stücken gesucht werden soll
) -> Option<(Cheese, PrevPieces)> {
    let mut i = 0; // Die aktuelle Länge der Pfade
    while !top_paths.is_empty() {
        if i == min_path_len / 2 && i > 3 && find_missing {
            // Wenn etwas Zeit vergangen ist, wird nur der Pfad mit den wenigsten
            // hinzugefügten Stücken weiterverfolgt
            top_paths = filter_top_paths(top_paths);
        }
        let mut new_top_paths = vec![];
        for sub_paths in top_paths {
            let mut new_paths = vec![];
            // Der aktuelle Pfad wird gespeichert, sodass er als Ergebnis zurückgegeben werden kann,
            // wenn keine neuen Pfade gefunden werden
            let curr_result = {
                let PossPath {
                    cheese,
                    prev_pieces: path,
                    ..
                } = &sub_paths[0];
                (*cheese, path.clone())
            };
            //erzeugt neue Pfade
            for poss_path in sub_paths.into_iter() {
                let paths = poss_path.gen_new_paths(find_missing);
                new_paths.extend(paths);
            }
            // Entfernt Pfade mit mehr hinzugefügten Stücken
            if new_paths.len() > 1 {
                new_paths = filter_sub_paths(new_paths);
            }
            if new_paths.is_empty() {
                if i >= min_path_len {
                    return Some(curr_result);
                }
            } else {
                // Entfernt gleiche Pfade
                new_top_paths.push(new_paths);
            }
        }
        top_paths = new_top_paths;
        i += 1;
    }
    None
}
/// Findet alle möglichen Käse
pub fn construct_cheeses(
    pieces: Box<PiecesMap>,
    // Anzahl der Stücke, wird benötigt da gleiche Stücke
    // im Pieces-Objekt zusammengefasst werden
    n_pieces: usize,
    find_missing: bool,
) -> Vec<(Cheese, PrevPieces)> {
    let mut pieces_map = pieces;
    // Die gefundenen Käse
    let mut results = vec![];
    // Die Stücke, die bereits verwendet wurden
    // Wird verwendet um zu überprüfen ob alle Stücke verwendet wurden
    let mut used_pieces = vec![];
    let mut min_path_len = n_pieces * 3 / 4; // Die minimale Länge eines Pfades
                                             // Es wird davon ausgegangen, dass ein Käse mindestens 3 Stücke hat
    while min_path_len > (n_pieces - used_pieces.len()) / 5 {
        // Die Stücke, die noch nicht verwendet wurden
        // werden als Startstücke verwendet
        let keys = pieces_map.base.keys().cloned().collect::<Vec<Piece>>();
        let top_paths = keys
            .iter()
            .map(|piece| {
                let cheese = Cheese::new([piece.0, piece.1, 0]);
                let start = PossPath::new(cheese, PrevPieces::new(*piece), pieces_map.clone());
                vec![start]
            })
            .collect::<Vec<_>>();
        // Es wird versucht einen Käse zu finden
        if let Some((cheese, path)) = construct_cheese(top_paths, min_path_len, find_missing) {
            // Wenn ein Käse gefunden wurde, werden die Stücke aus dem Pieces-Objekt entfernt
            let new_used_pieces = path.curr.get_real_pieces();
            pieces_map = Box::new(pieces_map.clone_without(&new_used_pieces));
            used_pieces.extend(new_used_pieces);

            // Die minimale Pfadlänge wird angepasst
            min_path_len = (n_pieces - used_pieces.len()) * 3 / 4;
            results.push((cheese, path));
            // found_cheese = true;
        } else {
            // Es wurde mit der der aktuellen Mindestlänge kein Käse gefunden,
            // Weshalb die Mindestlänge halbiert wird
            min_path_len /= 2;
        }
    }
    if !results.is_empty() {
        // Es wird überprüft ob alle Stücke verwendet wurden
        match used_pieces.len().cmp(&n_pieces) {
            Ordering::Less => {
                panic!(
                    "not all pieces used!! pieces left: {}",
                    pieces_map.base.values().sum::<u32>()
                );
            }
            Ordering::Greater => {
                panic!("error, used {} out of {}", used_pieces.len(), n_pieces);
            }
            Ordering::Equal => {}
        }
    }
    results
}
