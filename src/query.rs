use crate::RellTree;
use crate::binding::*;

pub struct QueryState
{
    binding_state: BindingState,
}

impl QueryState
{
    pub fn new() -> Self
    {
        Self
        {
            binding_state: BindingState::new(),
        }
    }

    pub fn from_statement<S>(query: &S) -> Self where S: AsRef<str>
    {
        let mut binding_state = BindingState::new();

        binding_state.add_statement(query);

        Self
        {
            binding_state
        }

    }
}

pub fn query_on<S>(query: S, tree: &RellTree) -> Vec<String> where S: AsRef<str>
{
    let mut q_state = QueryState::from_statement(&query); 
    q_state.binding_state.generate_compatible_on(tree);
    q_state.binding_state.get_all_bound_paths_for(&query)
}

#[cfg(test)]
mod test
{
    use super::*;
    use crate::errors::*;

    fn build_test_tree() -> Result<RellTree>
    {
        let mut w = RellTree::new();
        w.add_statement("city.in.state")?;
        w.add_statement("state.in.country")?;
        w.add_statement("other_state.in.country")?;
        w.add_statement("nothing.important")?;
        w.add_statement("something.in")?;

        Ok(w)
    }

    #[test]
    fn test() -> Result<()>
    {
        let w = build_test_tree()?;
        let q_result = query_on("X.in.state", &w);
        assert_eq!(q_result[0], "city.in.state", "Query didnt result expected value");

        let q_result_2 = query_on("X.in.Y", &w);
        assert_eq!(q_result_2.len(), 3, "Incorrect number of results for bound query");
        assert_eq!(q_result_2[0], "state.in.country", "Query didnt result expected value");
        assert_eq!(q_result_2[1], "city.in.state", "Query didnt result expected value");
        assert_eq!(q_result_2[2], "other_state.in.country", "Query didnt result expected value");

        Ok(())
    }
}
