use crate::rellcore::errors::*;
use crate::tree::*;
use crate::binding::*;

pub mod implications
{
    use super::*;

    pub struct Implication
    {
        pub prior: RellTree,
        pub posterior: RellTree 
    }

    impl Implication
    {
        pub fn from_statements<S>(priors: Vec<S>, posteriors: Vec<S>) -> Result<Self>
          where S: AsRef<str>
        {
            let mut prior = RellTree::new(); 
            for stmnt in priors
            {
                prior.add_statement(stmnt)?;
            }

            let mut posterior = RellTree::new(); 
            for stmnt in posteriors
            {
                posterior.add_statement(stmnt)?;
            }

            Ok(Self { prior, posterior })
        }

        pub fn apply(&self, tree: &mut RellTree) -> bool
        {
            if *tree < self.prior 
            {
                *tree = tree.greatest_lower_bound(&self.posterior).unwrap();
                true
            }
            else
            {
                // Implication did not trigger
                false
            }
        }
    }

    pub struct BindableImplication
    {
        pub binding_state: BindingState,
        pub posteriors: Vec<String>
    }

    impl BindableImplication
    {
        pub fn from_statements<S>(priors: Vec<S>, posteriors: Vec<S>) -> Result<Self>
          where S: AsRef<str>
        {
            let mut binding_state = BindingState::new();

            for prior in priors
            {
                binding_state.add_statement(prior);
            }

            let posteriors = posteriors.iter().map( | s | s.as_ref().to_string() ).collect();

            Ok(Self { binding_state, posteriors })
        }

        pub fn apply(&mut self, tree: &mut RellTree) -> Result<bool>
        {
            self.binding_state.bind_all(&tree);

            let mut compat_bindings = self.binding_state.generate_compatible();

            if compat_bindings.is_empty()
            {
                Ok(false)
            }
            else
            {
                tree.symbols.bind_variables(&mut compat_bindings[0]); // TODO: What to do with multiple?
                for posterior in &self.posteriors
                {
                    tree.add_statement(posterior)?;
                }
                tree.symbols.clear_bindings();
                Ok(true)
            }
            
        }

    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use super::implications::*;

    #[test]
    fn test_apply() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("city.in.state")?;
        t.add_statement("state.in.country")?;

        let imp = implications::Implication::from_statements(vec!["city.in.state", "state.in.country"], vec!["city.in.country"])?;

        assert!(imp.apply(&mut t), "Transitive implication did not trigger!");

        assert_eq!(format!("{}", t),
        "ROOT\n\
         -city\n\
         --in\n\
         ---country\n\
         ---state\n\
         -state\n\
         --in\n\
         ---country\n");

        let imp2 = implications::Implication::from_statements(vec!["city.in.state", "state.in.country", "nope.nope"], vec!["city.in.country"])?;
        assert!(!imp2.apply(&mut t), "Implication triggered at incorrect time!");

        let imp3 = implications::Implication::from_statements(vec!["city.in.country"], vec!["city.is.countrian"])?;
        assert!(imp3.apply(&mut t), "Simple implication did not trigger!");
        assert_eq!(format!("{}", t),
        "ROOT\n\
         -city\n\
         --is\n\
         ---countrian\n\
         --in\n\
         ---country\n\
         ---state\n\
         -state\n\
         --in\n\
         ---country\n");

        Ok(())
    }

    #[test]
    fn bindable_logic() -> Result<()>
    {
        let mut w = RellTree::new(); 
        w.add_statement("city.in.state")?;
        w.add_statement("state.in.country")?;
        w.add_statement("other_state.in.country")?;

        let mut imp = BindableImplication::from_statements(
                            vec!["X.in.Y", "Y.in.Z"],    // Implication Priors
                            vec!["X.in.Z"])?;            // Posteriors

        assert!(matches!(imp.apply(&mut w), Ok(t) if t));
        assert!(w.query("city.in.country").is_some());   // Assert Posterior of Implication

        Ok(())
    }
}
