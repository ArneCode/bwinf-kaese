use std::{mem, rc::Rc};

use crate::cheese::Piece;

//Eine Art Linked-List für zu einem Käse hinzugefügte Scheiben
//Hier wird die Liste nicht als Liste implementiert, sondern als Baum
//Das heißt, dass mehrere Path-Instanzen auf Teile der selben Scheiben-Liste zeigen können
//(siehe Dokumentation)
#[derive(Clone)]
pub struct PrevPieces {
    pub curr: Rc<HistPoint>, //die Head-Node der Liste
    pub start_piece: Piece,  //die erste Scheibe die zum Käse hinzugefügt wurde
    pub len: usize,          //die Anzahl der Scheiben, die zum Käse hinzugefügt wurden (nur echte)
    //die Anzahl der hypothetischen fehlenden Scheiben,
    //die zum Käse hinzugefügt wurden
    pub n_added: u32,
}
impl PrevPieces {
    //erzeugt eine neue Instanz
    pub fn new(value: Piece) -> Self {
        Self {
            curr: Rc::new(HistPoint::new(value)),
            start_piece: value,
            len: 1,
            n_added: 0,
        }
    }
    ///erzeugt eine neue Instanz, mit einer zusätzlichen (echten) Scheibe
    pub fn extend_real(&self, value: Piece) -> Self {
        Self {
            curr: self.curr.make_next(value, false),
            start_piece: self.start_piece,
            len: self.len + 1, //die Länge der Liste wird um 1 erhöht
            n_added: self.n_added,
        }
    }
    ///erzeugt eine neue Instanz, mit einer zusätzlichen (hypothetischen) Scheibe
    pub fn extend_added(&self, value: Piece) -> Self {
        Self {
            curr: self.curr.make_next(value, true),
            start_piece: self.start_piece,
            len: self.len,             //die Länge der Liste bleibt gleich
            n_added: self.n_added + 1, //die Anzahl der hypothetischen Scheiben wird um 1 erhöht
        }
    }
}
//ein Knoten der Liste
#[derive(Debug, Clone)]
pub struct HistPoint {
    //der Vorgänger-Knoten wird als Rc gespeichert,
    //da mehrere Instanzen auf den selben Vorgänger-Knoten zeigen können
    prev: Option<Rc<HistPoint>>,
    //die Käsescheibe, die zum Käse hinzugefügt wurde
    value: Piece,
    //wenn true, handelt es sich um eine hypothetische Scheibe
    is_added: bool,
}

impl HistPoint {
    //erzeugt eine neue Instanz
    fn new(value: Piece) -> Self {
        Self {
            prev: None,
            value,
            is_added: false,
        }
    }
    //erzeugt eine neue Instanz, die auf den Vorgänger-Knoten zeigt
    fn make_next(self: &Rc<Self>, value: Piece, is_added: bool) -> Rc<Self> {
        Rc::new(Self {
            prev: Some(self.clone()),
            value,
            is_added,
        })
    }
    //gibt die Liste als Array zurück
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
    ///gibt die Liste als Array von echten Scheiben zurück,
    ///d.h. die hypothetischen Scheiben werden ausgelassen
    pub fn get_real_pieces(self: &Rc<HistPoint>) -> Vec<Piece> {
        self.to_array()
            .iter()
            .filter(|pt| !pt.is_added)
            .map(|pt| pt.value)
            .collect()
    }
    ///gibt die Liste als Array von Scheiben zurück,
    /// egal ob sie echte oder hypothetische Scheiben sind
    pub fn get_pieces(self: &Rc<HistPoint>) -> Vec<Piece> {
        self.to_array().iter().map(|pt| pt.value).collect()
    }
}
impl Drop for PrevPieces {
    //wird aufgerufen, wenn eine Instanz von Path gelöscht wird
    //ansonsten würde die Liste rekursiv gelöscht werden
    //was bei einer großen Länge zu einem (Call) Stack Overflow führen kann
    fn drop(&mut self) {
        let curr = if let Some(curr) = Rc::get_mut(&mut self.curr) {
            curr
        } else {
            return;
        };
        // Laufe durch die Liste und lösche alle Knoten
        // nachdem die prev-Referenz auf None gesetzt wurde

        let mut maybe_node = mem::take(&mut curr.prev);
        while let Some(mut node) = maybe_node {
            if let Some(node_mut) = Rc::get_mut(&mut node) {
                // setze die prev-Referenz auf None
                // und speichere den Vorgänger-Knoten
                // um ihn im nächsten Schritt zu löschen
                maybe_node = mem::take(&mut node_mut.prev);
                // Knoten geht out of scope und wird gelöscht
            } else {
                // am Ende der Liste angekommen
                return;
            }
        }
    }
}
