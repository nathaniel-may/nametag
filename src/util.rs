use std::iter::Peekable;

pub trait NametagIterExt<I> {
    // TODO make this lazy instead of strict
    fn drain_while<P>(&mut self, p: P) -> Vec<I>
    where
        P: Fn(&I) -> bool;
}

impl<I, T> NametagIterExt<I> for Peekable<T>
where
    I: Clone,
    T: Iterator<Item = I>,
{
    fn drain_while<P>(&mut self, p: P) -> Vec<I>
    where
        P: Fn(&I) -> bool,
    {
        let mut result = vec![];

        loop {
            match self.peek() {
                None => return result,
                Some(i) => {
                    if p(i) {
                        result.push(i.clone());
                        let _ = self.next();
                    } else {
                        return result;
                    }
                }
            }
        }
    }
}

#[test]
fn drain_while() {
    let mut init = [1, 2, 3, 4, 5].into_iter().peekable();
    let drained = init.drain_while(|x| x < &3);
    assert_eq!(vec![1, 2], drained);
    assert_eq!(vec![3, 4, 5], init.collect::<Vec<_>>());
}
