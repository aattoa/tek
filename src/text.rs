#![allow(dead_code)]

// Potential optimizations:
// - zero-width pieces
// - single buffer table
// - avoid find_piece traversal

#[derive(Clone, Copy, Debug)]
struct Piece {
    offset: usize,
    width: usize,
    kind: PieceKind,
}

#[derive(Clone, Copy, Debug)]
enum PieceKind {
    Original,
    Append,
}

#[derive(Clone, Copy, Debug)]
struct PiecePosition {
    piece_index: usize,
    relative_offset: usize,
}

#[derive(Default)]
pub struct PieceTable {
    original: String,
    append: String,
    pieces: Vec<Piece>,
}

impl Piece {
    fn split(self, relative_offset: usize) -> (Piece, Piece) {
        let left = Piece { width: relative_offset, ..self };
        let right = Piece {
            offset: self.offset + relative_offset,
            width: self.width - relative_offset,
            kind: self.kind,
        };
        (left, right)
    }
}

impl PieceTable {
    fn find_piece(pieces: &[Piece], offset: usize) -> Option<PiecePosition> {
        let mut current_offset = 0;
        for (piece_index, piece) in pieces.iter().enumerate() {
            current_offset += piece.width;
            if current_offset >= offset {
                return Some(PiecePosition {
                    piece_index,
                    relative_offset: piece.width - (current_offset - offset),
                });
            }
        }
        None
    }

    fn splice(&mut self, from: usize, to: usize, replacement: impl Iterator<Item = Piece>) {
        self.pieces.splice(from..=to, replacement.filter(|piece| piece.width != 0));
    }

    fn add_piece(&mut self, string: &str) -> Piece {
        let offset = self.append.len();
        self.append.push_str(string);
        Piece { offset, width: string.len(), kind: PieceKind::Append }
    }

    pub fn insert(&mut self, offset: usize, string: &str) {
        let position = PieceTable::find_piece(&self.pieces, offset).unwrap();
        let piece = self.pieces[position.piece_index];
        let new = self.add_piece(string);
        if position.relative_offset == 0 {
            self.pieces.insert(position.piece_index, new);
        }
        else if position.relative_offset == piece.width {
            self.pieces.insert(position.piece_index + 1, new);
        }
        else {
            let (left, right) = piece.split(position.relative_offset);
            self.splice(position.piece_index, position.piece_index, [left, new, right].into_iter());
        }
    }

    pub fn remove(&mut self, offset: usize, width: usize) {
        if width == 0 {
            return; // Avoid unnecessary piece splitting
        }
        let start = PieceTable::find_piece(&self.pieces, offset).unwrap();
        let stop = PieceTable::find_piece(&self.pieces, offset + width).unwrap();
        let (l, _) = self.pieces[start.piece_index].split(start.relative_offset);
        let (_, r) = self.pieces[stop.piece_index].split(stop.relative_offset);
        self.splice(start.piece_index, stop.piece_index, [l, r].into_iter());
    }

    fn string_for(&self, piece: Piece) -> &str {
        let buffer = match piece.kind {
            PieceKind::Original => &self.original,
            PieceKind::Append => &self.append,
        };
        &buffer[piece.offset..piece.offset + piece.width]
    }

    pub fn gather(&self) -> String {
        self.pieces.iter().map(|&piece| self.string_for(piece)).collect()
    }
}

impl From<String> for PieceTable {
    fn from(string: String) -> PieceTable {
        let piece = Piece { offset: 0, width: string.len(), kind: PieceKind::Original };
        PieceTable { original: string, append: String::new(), pieces: vec![piece] }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn piece_table() {
        let mut table: super::PieceTable = "helloworld".to_owned().into();
        assert_eq!(table.pieces.len(), 1);
        assert_eq!(table.original, "helloworld");
        assert_eq!(table.gather(), "helloworld");

        table.insert(5, ", ");
        assert_eq!(table.append, ", ");
        assert_eq!(table.gather(), "hello, world");
        assert_eq!(table.pieces.len(), 3);

        table.insert(12, ") end");
        assert_eq!(table.append, ", ) end");
        assert_eq!(table.gather(), "hello, world) end");
        assert_eq!(table.pieces.len(), 4);

        table.insert(0, "begin (");
        assert_eq!(table.append, ", ) endbegin (");
        assert_eq!(table.gather(), "begin (hello, world) end");
        assert_eq!(table.pieces.len(), 5);

        table.insert(13, "[12589]");
        assert_eq!(table.append, ", ) endbegin ([12589]");
        assert_eq!(table.gather(), "begin (hello,[12589] world) end");
        assert_eq!(table.pieces.len(), 7);

        table.insert(16, "34");
        assert_eq!(table.append, ", ) endbegin ([12589]34");
        assert_eq!(table.gather(), "begin (hello,[1234589] world) end");
        assert_eq!(table.pieces.len(), 9);

        table.insert(19, "67");
        assert_eq!(table.append, ", ) endbegin ([12589]3467");
        assert_eq!(table.gather(), "begin (hello,[123456789] world) end");
        assert_eq!(table.pieces.len(), 11);

        table.remove(0, 3);
        assert_eq!(table.gather(), "in (hello,[123456789] world) end");
        assert_eq!(table.pieces.len(), 11);

        table.remove(2, 2);
        assert_eq!(table.gather(), "inhello,[123456789] world) end");
        assert_eq!(table.pieces.len(), 11);

        table.remove(0, 2);
        assert_eq!(table.gather(), "hello,[123456789] world) end");
        assert_eq!(table.pieces.len(), 10);

        table.remove(6, 11);
        assert_eq!(table.gather(), "hello, world) end");
        assert_eq!(table.pieces.len(), 5);

        table.remove(0, 12);
        assert_eq!(table.gather(), ") end");
        assert_eq!(table.pieces.len(), 1);

        table.remove(1, 0);
        assert_eq!(table.gather(), ") end");
        assert_eq!(table.pieces.len(), 1);

        table.remove(3, 1);
        assert_eq!(table.gather(), ") ed");
        assert_eq!(table.pieces.len(), 2);

        table.remove(0, 4);
        assert!(table.gather().is_empty());
        assert!(table.pieces.is_empty());

        assert_eq!(table.original, "helloworld");
        assert_eq!(table.append, ", ) endbegin ([12589]3467");
    }
}
