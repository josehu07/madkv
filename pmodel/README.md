# Consistency Models in [P Language](https://p-org.github.io/P/)

This is an attempt of trying out the cool P state machine modeling language for this [theory framework](https://arxiv.org/abs/2409.01576) of consistency levels.

Core definitions matching the paper can be found in `PSpec/Consistency.p`. Other files constitute a simple atomic object pool, which undergoes a linearizability check, to demonstrate how to produce ordering graphs by definition to aid consistency checking.

To run provided test case model checks:

```sh
p check -tc <TestCaseName> -s <NumSchedulesToTry>
```

For more rigorous specifications of consensus protocols using classic TLA+, take a look at [Summerset](https://github.com/josehu07/summerset).
