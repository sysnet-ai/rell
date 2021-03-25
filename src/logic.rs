use crate::rellcore::errors::*;
use crate::tree::*;

mod implications
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
}

#[cfg(test)]
mod test
{
    use super::*;

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
}
