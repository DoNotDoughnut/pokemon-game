use crate::{
    map::WorldMap,
    positions::{Coordinate, Destination, Direction, Path, Position},
};
use indexmap::{
    map::Entry::{Occupied, Vacant},
    IndexMap,
};
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    hash::Hash,
    ops::{Add, Sub},
};

pub fn pathfind(
    from: &Position,
    destination: Destination,
    player: Coordinate,
    world: &WorldMap,
) -> Option<Path> {
    let queue = astar(
        &(from.direction, from.coords),
        |n| valid_positions(n.1, world, &player),
        |p| distance(&p.1, &destination.coords),
        |p| destination.coords == p.1,
    )
    .map(|path| path.0.into_iter().map(|(dir, _)| dir).collect());
    queue.map(|queue| Path {
        queue,
        turn: destination.direction,
    })
}

fn valid_positions(
    coordinate: Coordinate,
    world: &WorldMap,
    player: &Coordinate,
) -> Vec<((Direction, Coordinate), isize)> {
    Direction::DIRECTIONS
        .iter()
        .map(|direction| (*direction, coordinate + direction.tile_offset()))
        .filter(|(_, coords)| {
            if world.in_bounds(*coords) && coords != player {
                world
                    .local_movement(*coords)
                    .map(can_walk)
                    .unwrap_or_default()
            } else {
                false
            }
        })
        .map(|pos| (pos, 1))
        .collect()
}

fn distance(coordinate: &Coordinate, other: &Coordinate) -> isize {
    (absdiff(coordinate.x, other.x) + absdiff(coordinate.y, other.y)) as isize
}

fn astar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Default + Add<Output = C> + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: Default::default(),
        cost: Default::default(),
        index: 0,
    });
    let mut parents: IndexMap<N, (usize, C)> = IndexMap::new();
    parents.insert(start.clone(), (usize::max_value(), Default::default()));
    while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap();
            if success(node) {
                let path = reverse_path(&parents, |&(p, _)| p, index);
                return Some((path, cost));
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
            match parents.entry(successor) {
                Vacant(e) => {
                    h = heuristic(e.key());
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        h = heuristic(e.key());
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
    }
    None
}

struct SmallestCostHolder<K> {
    estimated_cost: K,
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestCostHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}

#[inline]
fn absdiff<T>(x: T, y: T) -> T
where
    T: Sub<Output = T> + PartialOrd,
{
    if x < y {
        y - x
    } else {
        x - y
    }
}

fn reverse_path<N, V, F>(parents: &IndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let path = unfold(start, |i| {
        parents.get_index(*i).map(|(node, value)| {
            *i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();
    // Collecting the going through the vector is needed to revert the path because the
    // unfold iterator is not double-ended due to its iterative nature.
    path.into_iter().rev().cloned().collect()
}

fn unfold<A, St, F>(initial_state: St, f: F) -> Unfold<St, F>
where
    F: FnMut(&mut St) -> Option<A>,
{
    Unfold {
        f,
        state: initial_state,
    }
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
struct Unfold<St, F> {
    f: F,
    /// Internal state that will be passed to the closure on the next iteration
    pub state: St,
}

impl<A, St, F> Iterator for Unfold<St, F>
where
    F: FnMut(&mut St) -> Option<A>,
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        (self.f)(&mut self.state)
    }
}

pub fn tenth_walkable_coord(world: &WorldMap) -> Option<Coordinate> {
    let mut count: u8 = 0;
    for (i, m) in world.movements.iter().copied().enumerate() {
        if can_walk(m) {
            count += 1;
            if count == 10 {
                return Some(Coordinate::new(
                    (i % world.width) as _,
                    (i / world.width) as _,
                ));
            }
        }
    }
    None
}
