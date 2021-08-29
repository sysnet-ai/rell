#![cfg_attr(feature = "cargo-clippy", allow(clippy::suspicious_else_formatting))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]

#[macro_use]
extern crate log;

pub mod rellcore;
use rellcore::*;

pub mod parser;

pub mod tree;
use crate::tree::*;

pub mod tree_traits;

pub mod binding;
pub mod logic;
pub mod query;
pub mod symbols;

#[cfg(test)]
mod tests
{
    use crate::logic::*;
    use crate::tree::*;
    use crate::rellcore::errors::*;


    struct RellRuntime
    {
        rules: Vec<implications::BindableImplication>,
        world_tree: RellTree,
    }


    impl RellRuntime
    {
        pub fn update(&mut self) -> Result<()>
        {
            let mut need_update;
            loop
            {
                debug!("Update Loop Starting");
                need_update = false;
                for implication in &mut self.rules
                {
                    if implication.apply(&mut self.world_tree)?
                    {
                        need_update = true;
                    }
                }
                if !need_update
                {
                    debug!("Update Loop Ending");
                    break;
                }
            }
            
            Ok(())
        }
    }

    #[test]
    fn test_runtime() -> Result<()>
    {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut w = RellTree::new();
        w.add_statement("place.in.city")?;
        w.add_statement("city.in.state")?;
        w.add_statement("state.in.country")?;
        w.add_statement("other_state.in.country")?;
        w.add_statement("nothing.important")?;
        w.add_statement("something.in")?;


        let imp = implications::BindableImplication::from_statements(
                            vec!["X.in.Y", "Y.in.Z"],    // Implication Priors
                            vec!["X.in.Z"])?;            // Posteriors

        let mut rr = RellRuntime { rules: vec![imp], world_tree: w }; 

        rr.update()?;

        assert!(rr.world_tree.get_at_path("place.in.state").is_some());   // Assert Posterior of Implication
        assert!(rr.world_tree.get_at_path("city.in.country").is_some());   // Assert Posterior of Implication
        assert!(rr.world_tree.get_at_path("place.in.country").is_some());   // Assert Posterior of Implication

        Ok(())
    }
}
