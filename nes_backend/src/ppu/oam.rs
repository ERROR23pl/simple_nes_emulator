use std::{fmt::Display, ops::{Index, IndexMut}};


#[derive(Debug, Clone, Copy, Default)]
pub struct OamEntry {
    pub y: u8,
    pub id: u8,
    pub attribute: u8,
    pub x: u8,
}

impl Display for OamEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:03}, {:03}) | id: {:02X} | attribute: {:02X}", self.x, self.y, self.id, self.attribute)
    }
}

impl OamEntry {
    pub fn invisible() -> Self {
        OamEntry {
            y: 0xFF,
            id: 0xFF,
            attribute: 0xFF,
            x: 0xFF,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAM {
    entries: [OamEntry; 64],
}

impl Default for OAM {
    fn default() -> Self {
        Self {
            entries: [OamEntry::default(); 64]
        }
    }
}

impl OAM {
    pub fn get_entry(&self, index: usize) -> &OamEntry {
        &self.entries[index]
    }

    pub fn get_entries(&self) -> &[OamEntry] {
        &self.entries
    }
}

impl Index<u8> for OAM {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        let (entry, field) = (index / 4, index % 4);

        match field {
            0 => &self.entries[entry as usize].y,
            1 => &self.entries[entry as usize].id,
            2 => &self.entries[entry as usize].attribute,
            3 => &self.entries[entry as usize].x,

            4.. => unreachable!("`index % 4` always matches `0..=3`"),
        }
    }
}

impl IndexMut<u8> for OAM {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        let (entry, field) = (index / 4, index % 4);

        match field {
            0 => &mut self.entries[entry as usize].y,
            1 => &mut self.entries[entry as usize].id,
            2 => &mut self.entries[entry as usize].attribute,
            3 => &mut self.entries[entry as usize].x,

            4.. => unreachable!("`index % 4` always matches `0..=3`"),
        }
    }
}
