use crate::geometry::triangulation::trapezoidation::{
    graph::{Graph, X},
    ids::Id,
};

pub fn update(_id_x: Id, x: X, graph: &mut Graph) {
    graph.get_mut(x.left).sink_mut().unwrap().right_segment = Some(x.segment);
    graph.get_mut(x.right).sink_mut().unwrap().left_segment = Some(x.segment);

    // TASK: Implement:
    //       - Remove upper/lower boundary, if bounding point is on wrong side
    //         of the new segment. Mark affected regions for merging.
    //       - Update boundaries of upper and lower neighbors accordingly.
    //       - Merge all regions marked for merging that have the same left/
    //         right segment.
}

#[cfg(test)]
mod tests {
    use crate::geometry::triangulation::trapezoidation::{
        graph, insert::segment, point::Point, segment::Segment,
    };

    use super::update;

    // Looks useless, but actually makes sure that our calls to `Graph::new`
    // pick up the default type parameters, without us having to add an
    // additional type hint.
    type Graph = graph::Graph;

    #[test]
    pub fn update_should_update_new_boundary() {
        let mut graph = Graph::new();

        let id_x = segment::insert(
            Segment::new(Point::new(0.0, 0.0), Point::new(0.0, 1.0)).unwrap(),
            &mut graph,
        )
        .pop()
        .unwrap();
        let x = graph.get(id_x).x().unwrap().clone();

        update(id_x, x.clone(), &mut graph);

        assert_eq!(
            graph.get(x.left).sink().unwrap().right_segment.unwrap(),
            x.segment
        );
        assert_eq!(
            graph.get(x.right).sink().unwrap().left_segment.unwrap(),
            x.segment
        );
    }
}
