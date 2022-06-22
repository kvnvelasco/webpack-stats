/*
 * Copyright [2022] [Kevin Velasco]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use clap::Parser;
use std::borrow::Cow;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use webpack_q::graphs::ser;
use webpack_q::graphs::ser::GraphSerialization;
use webpack_q::operations::{
    describe_chunk, describe_entrypoints, display_entrypoints, paths_to_chunk, traverse_entry_chunk,
};

pub use webpack_q::prelude::*;
use webpack_q::templating::write_html_files_to_directory;
use webpack_q::webpack_stats::chunk::ChunkId;
use webpack_q::webpack_stats::WebpackStats;

#[derive(Parser)]
struct Args {
    stats_file: PathBuf,
    #[clap(short)]
    quiet: bool,
    #[clap(subcommand)]
    command: Command,
}
#[derive(clap::Subcommand)]
enum Command {
    /// List out all the possible entrypoints by index. Pass the index to
    /// other commands to traverse entrypoint
    #[clap(name = "list-entrypoints")]
    ListEntrypoints,
    /// Show statistics and traversal of an entrypoint
    DescribeEntrypoint { entrypoint_name: String },

    /// Show information about a specific chunk
    DescribeChunk { chunk_id: u32 },
    /// From an entrypoint in list-entrypoints, output a full traversal of that entrypoint and output it
    #[clap(name = "traverse-entrypoint")]
    TraverseEntrypoint {
        entrypoint_name: String,
        #[clap(short = 'f', long, value_enum, default_value_t = Output::Json)]
        output_format: Output,
        #[clap(short = 'o', default_value = "webpack-q")]
        output_path: PathBuf,
    },
    /// Find all the possible ways that an entrypoint escapes into a target chunk.
    /// e.g. paths-to-chunk entry-chunk 6332
    /// where 6332 is your commons chunk
    #[clap(name = "paths-to-chunk")]
    PathsToChunk {
        entrypoint_name: String,
        chunk: u32,
        #[clap(short = 'f', value_enum, default_value_t = Output::Json)]
        output_format: Output,
        #[clap(short = 'o', default_value = "webpack-q")]
        output_path: PathBuf,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Output {
    Json,
    Html,
    Dot,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    tracing_subscriber::fmt::init();
    // assume v5 for now;
    let contents = {
        let mut file = OpenOptions::new().read(true).open(&args.stats_file)?;
        let mut reader = BufReader::new(&mut file);
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        string
    };

    let stats = webpack_q::webpack_stats::deserialize_any_version(&contents)?;

    match args.command {
        Command::ListEntrypoints => {
            let entrypoints = match &stats {
                WebpackStats::V5(stats) => stats.entrypoints.values().collect::<Vec<_>>(),
            };

            let display = display_entrypoints(&entrypoints);
            println!("{}", display);
        }
        Command::DescribeChunk { chunk_id } => {
            let chunk_id = ChunkId(chunk_id);
            let description = match stats {
                WebpackStats::V5(stats) => describe_chunk(chunk_id, &stats.chunks, &stats.modules),
            };
            if let Some(description) = description {
                println!("{}", description);
            }
        }
        Command::PathsToChunk {
            chunk,
            entrypoint_name,
            output_path,
            output_format,
        } => {
            let graph = match stats {
                WebpackStats::V5(stats) => {
                    let entrypoint = stats
                        .entrypoints
                        .get(&Cow::Owned(entrypoint_name))
                        .ok_or(anyhow::anyhow!("Entrypoint does not exist"))?;

                    let target_chunk = ChunkId(chunk);
                    paths_to_chunk(entrypoint, target_chunk, &stats.chunks, &stats.modules)
                }
            };
            output_graph(&output_path, &output_format, move |mut writer| {
                let _ = match &output_format {
                    Output::Json | Output::Html => {
                        let serializable = GraphSerialization::<_, ser::NodeEdge>::new(graph);
                        serde_json::to_writer_pretty(writer, &serializable);
                    }
                    Output::Dot => {
                        dot::render(graph.inner(), &mut writer);
                    }
                };
                Ok(())
            })?;
        }

        Command::TraverseEntrypoint {
            entrypoint_name,
            output_format,
            output_path,
        } => {
            let graph = match stats {
                WebpackStats::V5(stats) => {
                    let entrypoint = stats
                        .entrypoints
                        .get(&Cow::Owned(entrypoint_name))
                        .ok_or(anyhow::anyhow!("Entrypoint does not exist"))?;
                    traverse_entry_chunk(stats.modules, stats.chunks, entrypoint)?
                }
            };
            output_graph(&output_path, &output_format, move |mut writer| {
                let _ = match &output_format {
                    Output::Json | Output::Html => {
                        let serializable = GraphSerialization::<_, ser::NodeEdge>::new(graph);
                        serde_json::to_writer_pretty(writer, &serializable);
                    }
                    Output::Dot => {
                        dot::render(graph.inner(), &mut writer);
                    }
                };
                Ok(())
            })?;
        }
        Command::DescribeEntrypoint {
            entrypoint_name, ..
        } => {
            let description = match stats {
                WebpackStats::V5(stats) => {
                    let entrypoints = stats.entrypoints.values().collect::<Vec<_>>();
                    let entries = display_entrypoints(&entrypoints);
                    describe_entrypoints(stats.chunks, &entrypoint_name, entries)?
                }
            };
            println!("{}", description);
        }
    }

    Ok(())
}

fn output_graph(
    output_path: &Path,
    output_format: &Output,
    write: impl FnOnce(&mut BufWriter<&mut File>) -> std::io::Result<()>,
) -> anyhow::Result<()> {
    let output_path = {
        let mut output_path = output_path.to_path_buf();
        match output_format {
            Output::Json => {
                output_path.set_extension("json");
            }
            Output::Html => {
                output_path.set_extension("html");
            }
            Output::Dot => {
                output_path.set_extension("dot");
            }
        };

        output_path
    };

    match output_format {
        Output::Json | Output::Dot => {
            let mut logs_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&output_path)?;
            logs_file.set_len(0)?;

            let mut writer = BufWriter::new(&mut logs_file);
            write(&mut writer)?;
        }
        Output::Html => {
            create_dir_all(&output_path)?;
            write_html_files_to_directory(&output_path, write)?;
            tracing::info!(
                "Files outputted to {:?}. Open folder with a web server",
                &output_path
            )
        }
    }

    Ok(())
}
