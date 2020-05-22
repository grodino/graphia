use std::convert::TryFrom;
use std::io::Error;
use std::ops::Range;
use std::{fmt, fs};

#[derive(Debug)]
pub struct Contact {
    pub couple: (i32, i32),
    pub start: i32,
    pub end: i32,
}

/// Describe the format of list of contacts in a Graph
///
/// * `StartEnd`: each line follows the format n1 n2 ts te
/// * `CreateDelete`: a line `t n1 n2 C` for a contact creation and a line `t n1 n2 S` for contact suppression
pub enum GraphFileFormat {
    StartEnd,
    CreateDelete,
}

/// Describes a non stationnary Graph
///
/// contacts must be ordered by contact starting time.
#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<i32>,
    pub contacts: Vec<Contact>,
    pub duration: i32,
}

impl Default for Graph {
    /// Create an empty Graph by default
    fn default() -> Self {
        Graph {
            nodes: vec![],
            contacts: vec![],
            duration: 0,
        }
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({} nodes, {} time samples, {} contacts",
            self.nodes.len(),
            self.duration,
            self.contacts.len()
        )
    }
}

impl TryFrom<String> for Graph {
    type Error = &'static str;

    /// Read a graph from a String
    ///
    /// The string should be formatted as such : n1 n2 ts te\n ... where n1 and
    /// n2 are the identifiers of the two nodes involved in the contact,ts
    /// stands for the time at which the contact started, and te the time at
    /// which the last contact between n1 and n2 has been recorded. It is worth
    /// noticing that the contacts are undirected and that, by convention, n1 < n2
    fn try_from(s: String) -> Result<Graph, Self::Error> {
        // Convert lines into array of int
        let mut contacts: Vec<Contact> = s
            .lines()
            .map(|l| {
                l.split(" ")
                    .map(|s| s.parse::<i32>().expect("Parse error"))
                    .collect()
            })
            .map(|c: Vec<i32>| Contact {
                couple: (c[0], c[1]),
                start: c[2],
                end: c[3],
            })
            .collect();

        // Make the time start at zero
        let t_start: i32 = (&contacts).into_iter().map(|c| c.start).min().unwrap();

        // Change the time reference to start at 0
        for contact in &mut contacts {
            contact.start -= t_start;
            contact.end -= t_start;
        }
        let last_node = contacts.last().unwrap().couple.1;

        // Sort contacts by starting time
        contacts.sort_by(|a, b| a.start.cmp(&b.start));

        let mut g = Graph {
            nodes: Range {
                start: 1,
                end: last_node,
            }
            .collect(),
            contacts: contacts,
            ..Default::default()
        };
        g.update_duration();

        Ok(g)
    }
}

impl From<&Graph> for String {
    fn from(graph: &Graph) -> String {
        let contacts = graph
            .contacts
            .iter()
            .map(|c| format!("{} {} {} {} \n", c.couple.0, c.couple.1, c.start, c.end));

        let s: String = contacts.collect();
        return s;
    }
}

impl Graph {
    /// Read a graph from a file
    ///
    /// The file should be formatted as such : <n1 n2 ts te> where n1 and n2
    /// are the identifiers of the two nodes involved in the
    /// contact,ts stands for the time at which the contact started, and te the
    /// time at which the last contact between n1 and n2 has been recorded. It
    /// is worth noticing that the contacts are undirected and that, by
    /// convention, n1 < n2
    pub fn from_file(filename: &str) -> Result<Graph, Error> {
        let graph_string = fs::read_to_string(filename)?;

        let graph = Self::try_from(graph_string).unwrap();
        Ok(graph)
    }

    /// Convert to `String` following the `GraphFileFormat::StartEnd` format
    ///
    /// The algorithm has a time complexity of O(nlog(n)) where n is the number
    /// of contact
    fn to_create_delete(&self) -> String {
        let mut events: Vec<(i32, i32, i32, char)> = Vec::with_capacity(self.contacts.len());

        for contact in &self.contacts {
            events.push((contact.start, contact.couple.0, contact.couple.1, 'C'));
            events.push((contact.end + 1, contact.couple.0, contact.couple.1, 'S'));
        }
        events.sort_by(|e1, e2| e1.0.cmp(&e2.0));
        events
            .into_iter()
            .map(|e| format!("{} {} {} {}\n", e.0, e.1, e.2, e.3))
            .collect()
    }

    /// Save the graph to a file
    pub fn save(&self, filename: &str, file_format: GraphFileFormat) -> Result<(), Error> {
        match file_format {
            GraphFileFormat::StartEnd => {
                let text: String = self.into();
                fs::write(filename, text)?;
            }
            GraphFileFormat::CreateDelete => {
                let text = self.to_create_delete();
                fs::write(filename, text)?;
            }
        };

        Ok(())
    }

    /// Calculates the time that separate a contact from the next one involving
    /// the same pair of nodes
    ///
    /// Returns -1 if there is no more contact involving the same pair
    pub fn inter_contact(&self, contact_id: usize) -> i32 {
        let contact = &self.contacts[contact_id];

        for c in &self.contacts[contact_id..self.contacts.len()] {
            if c.start > contact.end
                && c.couple.0 == contact.couple.0
                && c.couple.1 == contact.couple.1
            {
                return c.start - contact.end;
            }
        }
        return -1;
    }

    /// Calculates the inter_contact histogram over the graph
    /// TODO: use better algo
    pub fn inter_contact_histo(&self) -> Vec<i32> {
        let mut histo: Vec<i32> = Vec::with_capacity(self.duration as usize);
        let mut inter_contact = Vec::with_capacity(self.contacts.len());

        for i in 0..self.contacts.len() {
            inter_contact.push(self.inter_contact(i as usize));
        }

        for x in inter_contact {
            if x >= 0 {
                while (x as usize) >= histo.len() {
                    histo.push(0);
                }

                histo[x as usize] += 1;
            }
        }

        histo
    }

    /// Compute average degree at each instant
    pub fn average_degrees(&self) -> Vec<f32> {
        let mut events: Vec<(i32, i32, i32, char)> = Vec::with_capacity(self.contacts.len());

        for contact in &self.contacts {
            events.push((contact.start, contact.couple.0, contact.couple.1, 'C'));
            events.push((contact.end + 1, contact.couple.0, contact.couple.1, 'S'));
        }

        events.sort_by(|e1, e2| e1.0.cmp(&e2.0));
        let mut avg_degrees: Vec<f32> = Vec::with_capacity(self.duration as usize);

        let mut avg_degree: f32 = 0.0;
        let n = self.nodes.len() as f32;
        let mut delta_edges: i32 = 0;
        let mut t = 0;

        for event in &events {
            while event.0 != t {
                avg_degree += 2.0 * (delta_edges as f32) / n;
                avg_degrees.push(avg_degree);

                delta_edges = 0;
                t += 1;
            }

            if event.3 == 'C' {
                delta_edges += 1;
            } else {
                delta_edges -= 1;
            }
        }

        avg_degrees
    }

    /// Compute the fraction of created links at each time step
    pub fn fraction_created_links(&self) -> Vec<f32> {
        let mut events: Vec<(i32, i32, i32, char)> = Vec::with_capacity(self.contacts.len());

        for contact in &self.contacts {
            events.push((contact.start, contact.couple.0, contact.couple.1, 'C'));
            events.push((contact.end + 1, contact.couple.0, contact.couple.1, 'S'));
        }

        events.sort_by(|e1, e2| e1.0.cmp(&e2.0));
        let mut fraction_created: Vec<f32> = Vec::with_capacity(self.duration as usize);

        let n: f32 = self.nodes.len() as f32;
        let mut deleted_edges: i32 = 0;
        let mut created_edges: i32 = 0;
        let mut n_links: i32 = 0;
        let mut t = 0;

        for event in &events {
            while event.0 != t {
                fraction_created.push(
                    (2.0 * (created_edges as f32)) / (n * (n - 1.0) - 2.0 * (n_links as f32)),
                );

                n_links += created_edges - deleted_edges;
                deleted_edges = 0;
                created_edges = 0;
                t += 1;
            }

            if event.3 == 'C' {
                created_edges += 1;
            } else {
                deleted_edges += 1;
            }
        }

        fraction_created
    }

    /// Compute the fraction of deleted links at each time step
    pub fn fraction_deleted_links(&self) -> Vec<f32> {
        let mut events: Vec<(i32, i32, i32, char)> = Vec::with_capacity(self.contacts.len());

        for contact in &self.contacts {
            events.push((contact.start, contact.couple.0, contact.couple.1, 'C'));
            events.push((contact.end + 1, contact.couple.0, contact.couple.1, 'S'));
        }

        events.sort_by(|e1, e2| e1.0.cmp(&e2.0));
        let mut fraction_deleted: Vec<f32> = Vec::with_capacity(self.duration as usize);
        fraction_deleted.push(-1.0);

        let mut deleted_edges: i32 = 0;
        let mut created_edges: i32 = 0;
        let mut n_links: i32 = -1;
        let mut t = 0;

        for event in &events {
            while event.0 != t {
                fraction_deleted.push(match n_links {
                    -1 => {
                        n_links = 0;
                        -1.0
                    },
                    0 => 0.0,
                    _ => (deleted_edges as f32) / (n_links as f32),
                });

                n_links += created_edges - deleted_edges;
                deleted_edges = 0;
                created_edges = 0;
                t += 1;
            }

            if event.3 == 'C' {
                created_edges += 1;
            } else {
                deleted_edges += 1;
            }
        }

        fraction_deleted
    }

    /// Calculates total duration of the graph observation
    fn update_duration(&mut self) {
        self.duration = self.contacts.iter().map(|c| c.end).max().unwrap()
    }
}
