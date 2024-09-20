pub trait Append<A> {
    type Out;
    fn append(self, a: A) -> Self::Out;
}

impl<A0, A1> Append<A1> for (A0,) {
    type Out = (A0, A1);
    fn append(self, a1: A1) -> Self::Out {
        let (a0,) = self;
        (a0, a1)
    }
}

impl<A0, A1, A2> Append<A2> for (A0, A1) {
    type Out = (A0, A1, A2);
    fn append(self, a2: A2) -> Self::Out {
        let (a0, a1) = self;
        (a0, a1, a2)
    }
}

impl<A0, A1, A2, A3> Append<A3> for (A0, A1, A2) {
    type Out = (A0, A1, A2, A3);
    fn append(self, a3: A3) -> Self::Out {
        let (a0, a1, a2) = self;
        (a0, a1, a2, a3)
    }
}

impl<A0, A1, A2, A3, A4> Append<A4> for (A0, A1, A2, A3) {
    type Out = (A0, A1, A2, A3, A4);
    fn append(self, a4: A4) -> Self::Out {
        let (a0, a1, a2, a3) = self;
        (a0, a1, a2, a3, a4)
    }
}

impl<A0, A1, A2, A3, A4, A5> Append<A5> for (A0, A1, A2, A3, A4) {
    type Out = (A0, A1, A2, A3, A4, A5);
    fn append(self, a5: A5) -> Self::Out {
        let (a0, a1, a2, a3, a4) = self;
        (a0, a1, a2, a3, a4, a5)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6> Append<A6> for (A0, A1, A2, A3, A4, A5) {
    type Out = (A0, A1, A2, A3, A4, A5, A6);
    fn append(self, a6: A6) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5) = self;
        (a0, a1, a2, a3, a4, a5, a6)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7> Append<A7> for (A0, A1, A2, A3, A4, A5, A6) {
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7);
    fn append(self, a7: A7) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8> Append<A8> for (A0, A1, A2, A3, A4, A5, A6, A7) {
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8);
    fn append(self, a8: A8) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9> Append<A9> for (A0, A1, A2, A3, A4, A5, A6, A7, A8) {
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9);
    fn append(self, a9: A9) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10> Append<A10>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)
{
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
    fn append(self, a10: A10) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11> Append<A11>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)
{
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
    fn append(self, a11: A11) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12> Append<A12>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)
{
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
    fn append(self, a12: A12) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13> Append<A13>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)
{
    type Out = (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
    fn append(self, a13: A13) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12) = self;
        (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14> Append<A14>
    for (A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13)
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
    );
    fn append(self, a14: A14) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14,
        )
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15> Append<A15>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
    );
    fn append(self, a15: A15) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15,
        )
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16> Append<A16>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
    );
    fn append(self, a16: A16) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16,
        )
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17> Append<A17>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
    );
    fn append(self, a17: A17) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17,
        )
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18>
    Append<A18>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
    );
    fn append(self, a18: A18) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18,
        )
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19>
    Append<A19>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
        A19,
    );
    fn append(self, a19: A19) -> Self::Out {
        let (a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18) =
            self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18,
            a19,
        )
    }
}

impl<
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
        A19,
        A20,
    > Append<A20>
    for (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
        A19,
    )
{
    type Out = (
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
        A17,
        A18,
        A19,
        A20,
    );
    fn append(self, a20: A20) -> Self::Out {
        let (
            a0,
            a1,
            a2,
            a3,
            a4,
            a5,
            a6,
            a7,
            a8,
            a9,
            a10,
            a11,
            a12,
            a13,
            a14,
            a15,
            a16,
            a17,
            a18,
            a19,
        ) = self;
        (
            a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18,
            a19, a20,
        )
    }
}
