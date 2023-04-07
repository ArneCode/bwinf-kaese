use crate::{path::Path, pieces_map::PiecesMap, Piece, PossPath};

pub struct NewSide {
    //neue Seite, die an das Käsestück angefügt wird
    pub side_n: usize,  //Seitennummer
    pub piece: Piece,   //Das Stück, das an die Seite angefügt wird
    pub is_added: bool, //wurde das Stück aufgegessen und ist hypothetisch?
}

impl NewSide {
    fn new(side_n: usize, piece: Piece, is_added: bool) -> Self {
        Self {
            side_n,
            piece,
            is_added,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Cheese {
    //a<=b<=c
    pub size: [u32; 3],
}

impl Cheese {
    ///erzeugt ein neues Käsestück
    pub fn new(size: [u32; 3]) -> Self {
        Self { size }
    }
    ///gibt die Seitenlängen zurück, die für die Erzeugung einer Seite verwendet werden
    pub fn get_sides_n() -> Vec<(usize, usize)> {
        vec![(1, 2), (0, 2), (0, 1)]
    }
    ///gibt die Seiten des Käsestücks zurück
    pub fn get_sides(&self) -> Vec<Piece> {
        Cheese::get_sides_n()
            .into_iter()
            .map(|(a, b)| Piece(self.size[a], self.size[b]))
            .collect()
    }
    ///fügt eine Scheibe zum Käse hinzu, indem eine Seite vergrößert wird
    fn expand_side(&self, n: usize) -> Cheese {
        let mut size = self.size;
        size[n] += 1;
        //sortiert die Seitenlängen, damit die Käsestücke eindeutig wiederfindbar sind
        size.sort_unstable_by_key(|size| std::cmp::Reverse(*size));
        Cheese { size }
    }
    ///findet fehlende Scheiben (siehe Dokumentation)
    fn find_missing(
        &self,
        updated_sides: Vec<bool>, //welche Seiten bereits vergrößert wurden
        pieces: &PiecesMap,       //welche Scheiben noch vorhanden sind
    ) -> Vec<NewSide> {
        //sucht in den verbleibenden Scheiben nach einer die zu einem vergrößerten Käsestück passt
        Cheese::get_sides_n() //welche Seitenlängen für vergrößert werden
            .into_iter()
            .enumerate() //seite, die in den Scheiben gesucht wird
            .flat_map(|(i_searched, (other_a, other_b))| {
                if updated_sides[other_a] && updated_sides[other_b] {
                    //falls beide Seiten bereits vergrößert wurden, kann es hier keine fehlende Scheibe geben
                    return vec![];
                }
                //Der Käse aus der Sicht der Scheibe die nach der fehlenden Scheibe kommt
                let len_x = self.size[other_a];
                let len_y = self.size[other_b];
                let len_z = self.size[i_searched];

                //möglicherweise fehlende Scheiben, sowie die gesuchte Scheibe danach
                let mut poss_missing = vec![];
                if !updated_sides[other_a] {
                    let missing = Piece(len_y, len_z);
                    let searched = Piece(len_x + 1, len_y);
                    poss_missing.push((other_a, missing, searched))
                }
                if !updated_sides[other_b] {
                    let missing = Piece(len_x, len_z);
                    let searched = Piece(len_x, len_y + 1);
                    poss_missing.push((other_b, missing, searched))
                }
                poss_missing
            })
            //gibt nur Scheiben zurück, die wirklich fehlen,
            //d.h. es wird auch die Scheibe gefunden, die danach kommt
            .filter_map(|(added_i, added, next)| {
                //sortiert Scheiben aus, für die es keine darauf folgende Scheibe gibt
                let n_pieces = pieces.get(&next)?;
                if n_pieces == &0 {
                    return None;
                }
                Some(NewSide::new(added_i, added, true))
            })
            .collect::<Vec<_>>()
    }
    //findet neue Seiten, die an den Käse angefügt werden können
    pub fn find_new_sides(&self, pieces: &PiecesMap) -> (Vec<bool>, Vec<NewSide>) {
        //Seiten des Käses die bereits gesehen wurden, um nicht doppelte Pfade zu erzeugen
        //falls zwei Seiten gleich sind, wird nur eine davon verwendet
        let mut sides_seen = vec![];
        //welche Seiten bereits vergrößert wurden
        //wird nur benötigt, wenn man bei der Suche nach fehlenden Scheiben auch sucht,
        //wenn Seiten gefunden wurden (also momentan nicht, villeicht aber in Zukunft)
        let mut updated_sides = vec![false; 3];
        //finde neue Seiten, die an den Käse angefügt werden können
        let new_sides = self
            .get_sides()
            .into_iter()
            .enumerate()
            .filter(|(_, side)| {
                //filtert gleiche Seiten des Käses aus, um doppelte Pfade zu vermeiden
                if sides_seen.iter().any(|other| other == side) {
                    false
                } else {
                    sides_seen.push(*side);
                    true
                }
            })
            .filter_map(|(i, side)| {
                //überprüft, ob die Scheibe die zur Seite passt vorhanden ist
                let n_pieces = pieces.get(&side)?;
                if n_pieces == &0 {
                    return None;
                }
                updated_sides[i] = true; //markiert die Seite als vergrößert
                                         //gibt die Seite zurück, die an den Käse angefügt werden kann
                Some(NewSide::new(i, side, false))
            })
            .collect::<Vec<_>>();
        (updated_sides, new_sides)
    }
    ///erzeugt mögliche Pfade, indem es die neuen Seiten an den Käse anfügt
    pub fn new_sides_to_poss_path(
        &self,
        sides: Vec<NewSide>,    //neue Seiten, die an den Käse angefügt werden
        path: Path,             //der Pfad, der bis hierher gefolgt wurde
        pieces: Box<PiecesMap>, //welche Scheiben noch vorhanden sind
    ) -> Vec<PossPath> {
        //wandle zuerst die überbleibenden Scheibenliste in einen Pointer um,
        //damit sie einmal weniger kopiert werden müssen (siehe Dokumentation)
        let pieces_ptr = Box::into_raw(pieces);
        let n_side = sides.len();
        sides
            .into_iter()
            .enumerate()
            .map(move |(i, new_side)| {
                //wenn es die letzte Kopie ist die verwendet wird, verwende die Scheibenliste,
                //ohne sie zu kopieren
                let mut pieces = if i == n_side - 1 {
                    unsafe { Box::from_raw(pieces_ptr) }
                } else {
                    //ansonsten kopiere die Liste
                    Box::new(unsafe { pieces_ptr.as_mut() }.unwrap().make_copy())
                };
                //erzeuge neuen Pfad und entferne die Scheibe aus der Liste
                let new_path = if new_side.is_added {
                    //Scheibe wurde hinzugefügt (wurde aufgegessen)
                    path.extend_added(new_side.piece) //füge sie dem Pfad hinzu
                } else {
                    //Scheibe ist echt
                    //entferne die Scheibe aus der Liste
                    let mut n_pieces = *pieces.get(&new_side.piece).unwrap();
                    n_pieces -= 1;
                    pieces.insert(new_side.piece, n_pieces);

                    path.extend_real(new_side.piece) //füge die Scheibe dem Pfad hinzu
                };
                //erzeuge neuen Käse
                let new_cheese = self.expand_side(new_side.side_n);
                //gebe neuen Pfad zurück
                PossPath::new(new_cheese, new_path, pieces)
            })
            .collect()
    }
    ///erzeugt mögliche neue Pfade
    pub fn gen_poss_paths(
        &self,
        path: Path,
        pieces: Box<PiecesMap>,
        find_missing: bool,
    ) -> Vec<PossPath> {
        //findet neue Seiten, die an den Käse angefügt werden können
        let (updated_sides, mut new_sides) = self.find_new_sides(&pieces);
        // sucht nach fehlenden Scheiben, falls keine neuen Seiten gefunden wurden und
        // es möglicherweise fehlende Scheiben gibt
        if new_sides.is_empty() && find_missing {
            new_sides = self.find_missing(updated_sides, &pieces);
        }
        if !new_sides.is_empty() {
            //erzeugt mögliche Pfade, indem es die neuen Seiten an den Käse anfügt
            self.new_sides_to_poss_path(new_sides, path, pieces)
        } else {
            vec![]
        }
    }
}
//erzeugt einen Käse aus einer Scheibe
//kann durch die into() Funktion genutzt werden
impl From<Piece> for Cheese {
    fn from(value: Piece) -> Self {
        Cheese::new([value.0, value.1, 1])
    }
}
