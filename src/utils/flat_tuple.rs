pub trait FlattenTuple {
    type Flatten;

    fn flatten(self) -> Self::Flatten;
}

impl<C1> FlattenTuple for (C1,) {
    type Flatten = (C1,);

    fn flatten(self) -> Self::Flatten {
        self
    }
}

impl<C1, C2> FlattenTuple for (C1, (C2,)) {
    type Flatten = (C1, C2);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0)
    }
}

impl<C1, C2, C3> FlattenTuple for (C1, (C2, (C3,))) {
    type Flatten = (C1, C2, C3);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0, self.1 .1 .0)
    }
}

impl<C1, C2, C3, C4> FlattenTuple for (C1, (C2, (C3, (C4,)))) {
    type Flatten = (C1, C2, C3, C4);

    fn flatten(self) -> Self::Flatten {
        (self.0, self.1 .0, self.1 .1 .0, self.1 .1 .1 .0)
    }
}

impl<C1, C2, C3, C4, C5> FlattenTuple for (C1, (C2, (C3, (C4, (C5,))))) {
    type Flatten = (C1, C2, C3, C4, C5);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
        )
    }
}

impl<C1, C2, C3, C4, C5, C6> FlattenTuple for (C1, (C2, (C3, (C4, (C5, (C6,)))))) {
    type Flatten = (C1, C2, C3, C4, C5, C6);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .0,
        )
    }
}

impl<C1, C2, C3, C4, C5, C6, C7> FlattenTuple for (C1, (C2, (C3, (C4, (C5, (C6, (C7,))))))) {
    type Flatten = (C1, C2, C3, C4, C5, C6, C7);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .0,
        )
    }
}

impl<C1, C2, C3, C4, C5, C6, C7, C8> FlattenTuple
    for (C1, (C2, (C3, (C4, (C5, (C6, (C7, (C8,))))))))
{
    type Flatten = (C1, C2, C3, C4, C5, C6, C7, C8);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .0,
        )
    }
}

impl<C1, C2, C3, C4, C5, C6, C7, C8, C9> FlattenTuple
    for (C1, (C2, (C3, (C4, (C5, (C6, (C7, (C8, (C9,)))))))))
{
    type Flatten = (C1, C2, C3, C4, C5, C6, C7, C8, C9);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .1 .0,
        )
    }
}

impl<C1, C2, C3, C4, C5, C6, C7, C8, C9, C10> FlattenTuple
    for (C1, (C2, (C3, (C4, (C5, (C6, (C7, (C8, (C9, (C10,))))))))))
{
    type Flatten = (C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);

    fn flatten(self) -> Self::Flatten {
        (
            self.0,
            self.1 .0,
            self.1 .1 .0,
            self.1 .1 .1 .0,
            self.1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .1 .0,
            self.1 .1 .1 .1 .1 .1 .1 .1 .1 .0,
        )
    }
}
