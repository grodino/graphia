use std::io::Error;

use gnuplot::{Color, Figure, AxesCommon};
use std::path::PathBuf;
use structopt::StructOpt;

use log::{info, debug};

mod graph;
use graph::Graph;

mod models;

/// Dynamic graphs analysis and simulation.
#[derive(Debug, StructOpt)]
#[structopt()]
struct Opt {
    /// Save analysis to provided folder path. Make sure to include a "/" at the end
    #[structopt(short, long)]
    save: Option<PathBuf>,

    /// Do not show the graphs
    #[structopt(short = "n", long)]
    no_show: bool,

    /// Where to truncate the inter-contacts histogram
    #[structopt(short, long, default_value = "0.01")]
    truncate: f32,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Analyse a given graph and display its main characteristics
    Analyse {
        /// Graph input file
        ///
        /// The file should be formatted as such : <n1 n2 ts te> where n1 and n2
        /// are the identifiers of the two nodes involved in the
        /// contact,  n1 < n2, ts stands for the time at which the contact started, and te the
        /// time at which the last contact between n1 and n2 has been recorded.
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },

    /// Generate a graph using an EdgeMarkovian model
    Simulate {
        /// Number of time steps to generate
        #[structopt(short = "D", long)]
        duration: i32,

        /// Number of nodes in the graph
        #[structopt(short, long)]
        n_nodes: i32,

        /// Creation probability
        #[structopt(short = "cp", long)]
        creation_probability: f32,

        /// Deletion probability
        #[structopt(short = "dp", long)]
        deletion_probability: f32,
    },

    /// Analyse a graph and compare it to it's modeled version using Edge-Markovian model
    Compare {
        /// Model to simulate and compare with the data
        ///
        /// Can be :
        /// * `1`: Edge Markovian model
        /// * `2`: Time Dependent Edge Markovian model
        /// * `3`: Time Dependent Edge Markovian model with delayed nodes
        #[structopt(long_help = "Can be : \n \
            \t * 1: Edge Markovian model \n \
            \t * 2: Time Dependent Edge Markovian model \n \
            \t * 3: Time Dependent Edge Markovian model with delayed nodes")]
        model: u8,

        /// Graph input file
        ///
        /// The file should be formatted as such : <n1 n2 ts te> where n1 and n2
        /// are the identifiers of the two nodes involved in the
        /// contact,  n1 < n2, ts stands for the time at which the contact started, and te the
        /// time at which the last contact between n1 and n2 has been recorded.
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    }
}


fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    // Display every log message
    std::env::set_var("RUST_LOG", "TRACE");
    pretty_env_logger::init();

    let truncate = opt.truncate;

    // let (mut histo_fig, mut frac_fig, mut degree_fig) = match opt.cmd {
    let mut figures: Vec<Figure> = match opt.cmd {
        Command::Analyse { file } => {
            let analyse = Graph::from_file(file.to_str().unwrap())?;

            analyse_graph(&analyse, "", opt.truncate)
        },
        Command::Simulate { duration, n_nodes, creation_probability, deletion_probability } => {
            let simulation: Graph = Graph::from(models::EdgeMarkovian {
                duration,
                number_of_nodes: n_nodes,
                creation_probability,
                deletion_probability,
            });

            analyse_graph(&simulation, "", opt.truncate)
        },
        Command::Compare { model, file } => {
            debug!("Analysing graph");
            let analyse: Graph = Graph::from_file(file.to_str().unwrap())?;

            let frac_created = analyse.fraction_created_links();
            let frac_deleted = analyse.fraction_deleted_links();

            let mut analyse_figs = analyse_graph(&analyse, "REAL GRAPH: ", opt.truncate);

            debug!("Creating model (can take a very long time)");
            let simulation: Graph;

            match model {
                1 => {
                    // Compute Evolving-EdgeMarkovian model parameters
                    let creation_probability = frac_created.iter().filter(|&x| x >= &0.0)
                        .sum::<f32>() / frac_created.len() as f32;
                    let deletion_probability = frac_deleted.iter().filter(|&x| x >= &0.0)
                        .sum::<f32>() / frac_deleted.len() as f32;

                    simulation = Graph::from(models::EdgeMarkovian {
                        duration: analyse.duration,
                        number_of_nodes: analyse.nodes.len() as i32,
                        creation_probability,
                        deletion_probability,
                    });
                },
                2 => {
                    // remove all "-1" in the data
                    let creation_probability = frac_created.iter()
                        .map(|&frac| 0f32.max(frac))
                        .collect();
                    let deletion_probability = frac_deleted.iter()
                        .map(|&frac| 0f32.max(frac))
                        .collect();

                    simulation = Graph::from(models::TimeDependentEdgeMarkovian {
                        duration: analyse.duration,
                        number_of_nodes: analyse.nodes.len() as i32,
                        creation_probability,
                        deletion_probability,
                    });
                },
                3 => {
                    // remove all "-1" in the data
                    let creation_probability = frac_created.iter()
                        .map(|&frac| 0f32.max(frac))
                        .collect();
                    let deletion_probability = frac_deleted.iter()
                        .map(|&frac| 0f32.max(frac))
                        .collect();

                    // Compute and truncate contacts histogram
                    let mut contacts_histogram: Vec<i32> = analyse.inter_contact_histo();
                    let max: f32 = *contacts_histogram.iter().max().unwrap_or(&0) as f32;

                    contacts_histogram = contacts_histogram.into_iter()
                        .filter(|&x| x >= (truncate * max) as i32)
                        .collect();

                    simulation = Graph::from(models::DelayedTimeDependentEdgeMarkovian {
                        duration: analyse.duration,
                        number_of_nodes: analyse.nodes.len() as i32,
                        creation_probability,
                        deletion_probability,
                        intercontacts_histogram: contacts_histogram,
                    });
                }
                _ => unimplemented!()
            }

            info!("Analysing model");
            let mut model_figs = analyse_graph(&simulation, "MODEL: ", opt.truncate);
            analyse_figs.append(&mut model_figs);

            analyse_figs
        }
    };

    match opt.save {
        Some(destination) => {
            if destination.is_dir() == false {
                std::fs::create_dir(&destination)?;
            }

            for (i, figure) in figures.iter_mut().enumerate() {
                let mut path = PathBuf::from(&destination);
                path.push(format!("figure_{}.png", i));

                debug!("save file : {}", path.to_str().unwrap());

                figure.save_to_png(
                    path.to_str().unwrap(),
                    1000, 666
                ).unwrap();
            }
        },
        None => {}
    }

    if opt.no_show == false {
        for figure in figures.iter_mut() {
            figure.show().expect("Could not show figure");
        }
    }

    Ok(())
}

/// Analyse a graph and plot its analysed properties. Helper function, not meant to be reused in an
/// other context
fn analyse_graph(g: &Graph, title_prefix: &str, truncate: f32) -> Vec<Figure>{
    info!("number of nodes: {}", g.nodes.len());
    info!("number of contacts: {}", g.contacts.len());
    info!("duration: {}", g.duration);

    // Compute and truncate contacts histogram
    let mut contacts_histogram: Vec<i32> = g.inter_contact_histo();
    let max: f32 = *contacts_histogram.iter().max().unwrap_or(&0) as f32;

    contacts_histogram = contacts_histogram.into_iter()
        .filter(|&x| x >= (truncate * max) as i32)
        .collect();

    // Diplay contacts histogram
    let mut histo_fig = Figure::new();
    histo_fig.axes2d()
        .boxes(&mut(0..contacts_histogram.len()), &contacts_histogram, &[Color("black")])
        .set_y_label("number of inter-contacts", &[])
        .set_x_label("inter-contact duration (in sample)", &[]);
    histo_fig.set_title(
        format!(
            "{}Inter-contacts histogram (truncated to {}% of max intercontact)",
            title_prefix,
            (truncate * 100.0) as u8
        ).as_str()
    );

    // Compute and display fraction of created and deleted links
    let frac_created = g.fraction_created_links();
    let frac_deleted = g.fraction_deleted_links();

    let mut frac_fig = Figure::new();
    frac_fig.set_multiplot_layout(2, 1)
        .set_title(
            format!("{}Fractions of created and deleted edges", title_prefix).as_str()
        );

    frac_fig.axes2d()
        .points(&mut(0..frac_created.len()), &frac_created, &[Color("black")])
        .set_x_label("time (in sample)", &[])
        .set_y_label("fraction of created edges", &[]);
    frac_fig.axes2d()
        .points(&mut(0..frac_deleted.len()), &frac_deleted, &[Color("black")])
        .set_x_label("time (in sample)", &[])
        .set_y_label("fraction of deleted edges", &[]);

    // Compute and display average degree
    let avg_degree = g.average_degrees();
    let mut degree_fig = Figure::new();
    degree_fig.set_title(
        format!("{}Average degree over time", title_prefix).as_str()
    );

    degree_fig.axes2d()
        .points(&mut(0..avg_degree.len()), &avg_degree, &[Color("black")])
        .set_x_label("time (in sample)", &[])
        .set_y_label("average degree", &[]);

    // Compute Evolving-EdgeMarkovian model parameters
    let creation_probability = frac_created.iter().filter(|&x| x >= &0.0)
        .sum::<f32>() / frac_created.len() as f32;
    let deletion_probability = frac_deleted.iter().filter(|&x| x >= &0.0)
        .sum::<f32>() / frac_deleted.len() as f32;
    info!("average creation probability {}", creation_probability);
    info!("average deletion probability {}", deletion_probability);

    vec![histo_fig, frac_fig, degree_fig]
}