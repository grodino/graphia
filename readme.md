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
given a graph at a given time, the structure of the graph at the next time step depends directly on it. The stucture 
change is ruled by two independent parameters : the  probability that a new link is created and the probability that an
 existing link is deleted.

## Installation

### Use precompiled binary
Go to the [release page](https://github.com/grodino/graphia/releases) of the repository and download the binary 
corresponding to your system.

#### Windows
Just run the binary : 
```shell script
graphia.exe --help
```

#### Linux
Set the file permissions as executable : `chmod +x graphia-linux` and run it :
```shell script
./graphia-linux --help
```

#### MacOs
A binary is provided but not tested


### Install via Cargo 
This is the most reliable method. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, clone the 
repository and run cargo.

Once Rust is installed, run :
```shell script
git clone https://github.com/grodino/graphia.git
cd graphia
cargo run -- --help
```

> The `--` after `cargo run` are here to pass the command line arguments to `graphia` after it has been built by `cargo`.


## Examples

The CLI tool is subdivised in commands, you can list them via ```graphia -``` (or ```cargo run -- -``` if you
 use cargo). 
 
 To see the global help, run `graphia --help`. To see the help associated to a command, run 
 ```graphia <command> --help```.
 
 From now one, lets assume that you have a folder `data` at the same level as `graphia` and that it contains two 
 dataset files: `Rollernet` and `Infocom06`
  
 To display the analyse of a dataset an save it in a directory named `generated`, run :
 ```shell script
graphia --save generated/ analyse data/Rollernet
```
> Do not forget the slash at the end of the folder

To compare a dataset and the results of a simulation based on EdgeMarkovian from this dataset, save graphs in 
folder `generated`, and not plot the results in new windows, run :
```shell script
graphia --no-show --save generated/ compare 1 data/Rollernet
```