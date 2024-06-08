pub trait NametagIterExt<I> {
    // TODO make this lazy instead of strict
    fn drain_while<P>(&mut self, p: P) -> Vec<I>
    where
        P: Fn(&I) -> bool;
}

impl<I, T> NametagIterExt<I> for T
where
    I: Clone,
    T: Iterator<Item = I>,
{
    fn drain_while<P>(&mut self, p: P) -> Vec<I>
    where
        P: Fn(&I) -> bool,
    {
        let mut result = vec![];
        let mut cur = self.next();

        loop {
            match &cur {
                None => return result,
                Some(i) => {
                    if p(i) {
                        result.push(i.clone());
                        cur = self.next();
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
    let drained = [1, 2, 3, 4, 5].into_iter().drain_while(|x| x < &3);
    assert_eq!(vec![1, 2], drained);
}
