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
            loop
            {
                debug!("Update Loop Starting");
                if !self.step()?
                {
                    debug!("Update Loop Ending");
                    break;
                }
            }
            
            Ok(())
        }

        pub fn step(&mut self) -> Result<bool>
        {
            let mut need_update = false;
            for implication in &mut self.rules
            {
                need_update |= implication.apply(&mut self.world_tree)?;
            }
            Ok(need_update)
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

    #[test]
    fn test_goat() -> Result<()>
    {
        let _ = env_logger::builder().is_test(true).try_init();
        
        let mut w = RellTree::new();
        w.add_statement("goat.in!left")?;
        w.add_statement("cabagge.in!left")?;
        w.add_statement("dog.in!left")?;
        w.add_statement("man.in!left")?;

        // If goat and cabbage in the same side, and man on the other
        let goat_imp = implications::BindableImplication::from_statements(
                                vec!["goat.in!X", "cabagge.in!X", "man.in!Y"],
                                vec!["cabagge.is!eaten"])?;


        // If goat and dog in the same side, and man on the other
        let dog_imp = implications::BindableImplication::from_statements(
                                vec!["dog.in!X", "goat.in!X", "man.in!Y"],
                                vec!["goat.is!eaten"])?;

        // Moving while holding something should move the thing
        let mov_imp = implications::BindableImplication::from_statements(
                            vec!["man.holds!X", "man.in!Z"],
                            vec!["X.in!Z"])?;

        let mut rr = RellRuntime { rules: vec![mov_imp, goat_imp, dog_imp], world_tree: w }; 
        rr.update()?;

        // Everyone is A-OK
        assert!(rr.world_tree.get_at_path("goat.is!eaten").is_none());
        assert!(rr.world_tree.get_at_path("cabagge.is!eaten").is_none());

        // Man grabs goat and moves to the right
        rr.world_tree.add_statement("man.holds!goat")?;
        rr.world_tree.add_statement("man.in!right")?;
        rr.update()?;

        // Goat is right
        assert!(rr.world_tree.get_at_path("goat.in!right").is_some());
        // Everyone is alive
        assert!(rr.world_tree.get_at_path("goat.is!eaten").is_none());
        assert!(rr.world_tree.get_at_path("cabagge.is!eaten").is_none());

        // Man moves back... Still holds goat 
        rr.world_tree.add_statement("man.in!left")?;
        rr.update()?;

        // Goat back left
        assert!(rr.world_tree.get_at_path("goat.in!left").is_some());

        // Grab cabbage and move
        rr.world_tree.add_statement("man.holds!cabagge")?;
        rr.world_tree.add_statement("man.in!right")?;
        rr.update()?;

        // Goat is dead :( 
        assert!(rr.world_tree.get_at_path("goat.is!eaten").is_some());

        Ok(())
    }
}
