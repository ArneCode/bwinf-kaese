use rustc_hash::{FxHashMap, FxHasher};
use std::{hash::BuildHasherDefault, rc::Rc};
use uuid::Uuid;

use crate::cheese::Piece;

//Speichert die Käsescheiben, die noch über sind
#[derive(Debug, Clone)]
pub struct PiecesMap {
    //base-HashMap bleibt unverändert,
    //wird nicht geklont und zwischen verschiedenen Instanzen geteilt
    pub base: Rc<FxHashMap<Piece, u32>>,
    //base_id wird benutzt, um schnell zu überprüfen,
    //ob zwei Instanzen die selbe base-HashMap verwenden
    pub base_id: Uuid,
    //wird geklont und nicht zwischen verschiedenen Instanzen geteilt
    //added-HashMap wird bei Bedarf mit base-HashMap zusammengeführt
    pub added: FxHashMap<Piece, u32>,
}

impl PiecesMap {
    //erzeugt eine PiecesMap aus einer Liste von Käsescheiben
    pub fn new(pieces: &Vec<Piece>) -> PiecesMap {
        //die base-HashMap,
        //benuzt FxHashMap, da diese schneller ist als die Standard-HashMap
        let mut pieces_map: FxHashMap<Piece, u32> = FxHashMap::default();
        //die größte Anzahl derselben Käsescheibe
        let mut max_n = 0;
        //wie viele Käsescheiben mehrfach vorkommen
        let mut n_multiple = 0;
        for piece in pieces {
            //überprüft, ob die Käsescheibe schon in der HashMap ist
            if let Some(n) = pieces_map.get_mut(piece) {
                //wenn ja, wird die Anzahl der Scheibe in der HashMap um 1 erhöht
                *n += 1;
                n_multiple += 1;
                if n > &mut max_n {
                    max_n = *n;
                }
            } else {
                //wenn nicht, wird die Käsescheibe neu in die HashMap eingefügt
                pieces_map.insert(*piece, 1);
            }
        }
        //gebe Informationen über die Scheiben aus
        println!("Informationen über die Käsescheiben:");
        println!(
            "\tMaximale Anzahl eines einzelnen Stücks: {}\n\tMehrfache Scheiben: {}\n\tAnzahl verschiener Scheiben: {}",
            max_n,
            n_multiple,
            pieces_map.len()
        );
        println!();
        PiecesMap::new_from_map(pieces_map)
    }

    //erzeugt eine neue Instanz aus einer base-HashMap
    pub fn new_from_map(map: FxHashMap<Piece, u32>) -> Self {
        let base_id = Uuid::new_v4();
        Self {
            base: Rc::new(map),
            base_id,
            added: FxHashMap::default(),
        }
    }
    //gibt falls vorhanden die Anzahl der Käsescheiben zurück, die für k gefunden wurden
    pub fn get(&self, k: &Piece) -> Option<&u32> {
        if let Some(result) = self.added.get(k) {
            Some(result)
        } else {
            self.base.get(k)
        }
    }
    //fügt eine Käsescheibe hinzu oder verändert die Anzahl der Käsescheibe
    //es wird nur die added-HashMap verändert, um Zeit beim Klonen zu sparen
    pub fn insert(&mut self, k: Piece, v: u32) -> Option<u32> {
        self.added.insert(k, v)
    }
    //kombiniert die added-HashMap mit der base-HashMap zu einer neuen base-HashMap
    fn merge_hashmaps(&self) -> FxHashMap<Piece, u32> {
        //die neue base-HashMap
        let mut new_map = FxHashMap::with_capacity_and_hasher(
            self.base.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );
        //fügt alle Käsescheiben aus den beiden HashMaps zusammen
        for (k, v) in self.base.as_ref() {
            let v = if let Some(v) = self.added.get(k) {
                //falls ein Eintrag in der added-HashMap vorhanden ist, wird dieser verwendet
                *v
            } else {
                //ansonsten wird der Eintrag aus der base-HashMap verwendet
                *v
            };
            if v == 0 {
                //falls von einer Käsescheibe keine mehr vorhanden sind, wird sie übersprungen
                continue;
            }
            new_map.insert(*k, v);
        }
        new_map
    }
    //erzeugt eine neue Instanz,
    //in der die added-HashMap mit der base-HashMap zusammenführt wurde
    fn merge(&mut self) {
        let base = self.merge_hashmaps();
        self.base = Rc::new(base);
        self.added = FxHashMap::default();
    }
    //erzeugt eine neue Instanz, meistens einfach als Kopie
    //wenn die added-HashMap zu groß ist, wird sie mit der base-HashMap zusammengeführt
    pub fn make_copy(&mut self) -> Self {
        if self.added.len() > self.base.len() / 10 {
            //10% hat sich beim Ausprobieren als gute Größe herausgestellt
            self.merge();
        }
        self.clone()
    }
    //erzeugt eine neue Instanz ohne die Käsescheiben in removed
    //wird verwendet, um die Käsescheiben zu entfernen, die bereits verwendet wurden
    pub fn clone_without(&self, removed: &Vec<Piece>) -> Self {
        if removed.is_empty() {
            return self.clone();
        }
        //die neue base-HashMap, noch mit allen Käsescheiben
        let mut new_map = self.merge_hashmaps();
        //entfernt iterativ die Käsescheiben aus der neuen base-HashMap
        for piece in removed {
            //ob der Eintrag für die Käsescheibe entfernt werden soll
            let mut do_remove = false;
            if let Some(v) = new_map.get_mut(piece) {
                //entfernt eine Käsescheibe
                *v -= 1;
                if *v == 0 {
                    //wenn keine Käsescheibe mehr vorhanden ist, wird sie entfernt
                    do_remove = true;
                }
            }
            if do_remove {
                //entfernt den Eintrag für die Käsescheibe
                new_map.remove(piece);
            }
        }
        Self::new_from_map(new_map)
    }
}
