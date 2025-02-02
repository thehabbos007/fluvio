//! # List SPU Groups CLI
//!
//! CLI tree and processing to list SPU Groups
//!

use std::sync::Arc;
use clap::Parser;

use fluvio::Fluvio;
use fluvio_controlplane_metadata::spg::SpuGroupSpec;

use crate::cli::common::output::Terminal;
use crate::cli::common::OutputFormat;
use crate::cli::ClusterCliError;

#[derive(Debug, Parser)]
pub struct ListManagedSpuGroupsOpt {
    #[clap(flatten)]
    output: OutputFormat,
}

impl ListManagedSpuGroupsOpt {
    /// Process list spus cli request
    pub async fn process<O: Terminal>(
        self,
        out: Arc<O>,
        fluvio: &Fluvio,
    ) -> Result<(), ClusterCliError> {
        let admin = fluvio.admin().await;
        let lists = admin.list::<SpuGroupSpec, _>(vec![]).await?;

        output::spu_group_response_to_output(out, lists, self.output.format)
    }
}

mod output {

    //!
    //! # Fluvio SC - output processing
    //!
    //! Format SPU Group response based on output type

    use prettytable::Row;
    use prettytable::row;
    use prettytable::Cell;
    use prettytable::cell;
    use prettytable::format::Alignment;
    use tracing::debug;
    use serde::Serialize;

    use fluvio::metadata::objects::Metadata;
    use fluvio_controlplane_metadata::spg::SpuGroupSpec;

    use crate::cli::ClusterCliError;
    use crate::cli::common::output::{OutputType, TableOutputHandler, Terminal};
    use crate::cli::common::t_println;

    #[derive(Serialize)]
    struct ListSpuGroups(Vec<Metadata<SpuGroupSpec>>);

    // -----------------------------------
    // Format Output
    // -----------------------------------

    /// Format SPU Group based on output type
    pub fn spu_group_response_to_output<O: Terminal>(
        out: std::sync::Arc<O>,
        list_spu_groups: Vec<Metadata<SpuGroupSpec>>,
        output_type: OutputType,
    ) -> Result<(), ClusterCliError> {
        debug!("groups: {:#?}", list_spu_groups);

        if !list_spu_groups.is_empty() {
            let groups = ListSpuGroups(list_spu_groups);
            out.render_list(&groups, output_type)?;
            Ok(())
        } else {
            t_println!(out, "no groups");
            Ok(())
        }
    }

    // -----------------------------------
    // Output Handlers
    // -----------------------------------
    impl TableOutputHandler for ListSpuGroups {
        /// table header implementation
        fn header(&self) -> Row {
            row!["NAME", "REPLICAS", "MIN ID", "RACK", "SIZE", "STATUS",]
        }

        /// return errors in string format
        fn errors(&self) -> Vec<String> {
            self.0.iter().map(|_g| "".to_owned()).collect()
        }

        /// table content implementation
        fn content(&self) -> Vec<Row> {
            self.0
                .iter()
                .map(|r| {
                    let spec = &r.spec;
                    let storage_config = spec.spu_config.real_storage_config();
                    Row::new(vec![
                        Cell::new_align(&r.name, Alignment::RIGHT),
                        Cell::new_align(&spec.replicas.to_string(), Alignment::CENTER),
                        Cell::new_align(&r.spec.min_id.to_string(), Alignment::RIGHT),
                        Cell::new_align(
                            &spec.spu_config.rack.clone().unwrap_or_default(),
                            Alignment::RIGHT,
                        ),
                        Cell::new_align(&storage_config.size, Alignment::RIGHT),
                        Cell::new_align(&r.status.to_string(), Alignment::RIGHT),
                    ])
                })
                .collect()
        }
    }
}
