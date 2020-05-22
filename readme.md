# Graphia

## Introduction
`graphia` is a tool created for the "Machine Learning For Network Modeling" module of the ["parcours IA"](https://ens-paris-saclay.fr/etudes/diplome-ens-paris-saclay/parcours-intelligence-artificielle-ia).

The goal was to measure some properties of time-evolving graphs in order to create a suitable model for such graphs.

The first properties considered are :
* Average degree : the average degree of the node at each time sample
* Inter-contact histogram : the histogram of the time between two node come in contact againp
* Creation/deletion fraction : the number of created/deleted edges over the number of edges that could have been
 created/deleted

The first model used is the Edge-Markovian model :
given a graph at a given time, the structure of the graph at the next time step depends directly on it. The stucture change is ruled by two independent parameters : the  probability that a new link is created and the probability that an existing link is deleted.

## Installation

### Use precompiled binary
Go to the [release page](https://github.com/grodino/graphia/releases) of the repository and download the binary corresponding to your system.

#### Windows
Just run the binary : 
```cmd
graphia.exe -h
```

#### Linux
Set the file permissions as executable : `chmod +x graphia-linux` and run it :
```bash
./graphia-linux -h
```

#### MacOs
A binary is provided but not tested


### Install via Cargo 
This is the most reliable method. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, clone the repository and run cargo.

Once Rust is installed, run :
```cmd
git clone TODO
cargo run -- --help
```

> The `--` after `cargo run` are here to pass the command line arguments to `graphia` after it has been built by `cargo`.