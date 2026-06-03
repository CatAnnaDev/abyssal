# Abyssal pathfinders — C# port for Unity 6.0

A direct port of the Rust pathfinders in `../src/ai.rs`, engine-agnostic and allocation-free in the hot path.

## Files

- `Pathfinding.cs` — the core. `GridMap`, the `Pathfinder` enum (`Bfs`, `AStar`, `Dijkstra`, `Greedy`, `Diagonal`) and `PathSolver` with `StepTo`, `SearchCost`, `PathTo`, `BfsField`. No `UnityEngine` dependency, so it compiles anywhere.
- `PathfindingBenchmark.cs` — maze generator + `Stopwatch` benchmark reporting expanded nodes and µs/search per algorithm.
- `PathfindingBenchmarkRunner.cs` — `MonoBehaviour` wrapper, guarded by `#if UNITY_2020_1_OR_NEWER`. Right-click the component → **Run Benchmark** / **Demo Path**.

## Why a hand-rolled heap

Unity 6 targets .NET Standard 2.1, which has no `PriorityQueue<T,T>` (that arrived in .NET 6). `PathSolver` carries its own binary min-heap over packed `long` keys (`priority << 32 | index`), so ties break deterministically by cell index.

## Allocation-free reuse

`PathSolver` holds one pooled scratch (generation-stamped `seen`/`closed`/`g`/`came`/`block` arrays, a reusable heap and ring-buffer queue). Each search bumps a generation counter instead of clearing arrays, so after warm-up there are zero per-search allocations and no GC pressure. Create one `PathSolver` per thread; it is not thread-safe.

## Usage

```csharp
var walk = new bool[width * height];
var map = new GridMap(width, height, walk);
var solver = new PathSolver();

(int dx, int dy)? step = solver.StepTo(Pathfinder.AStar, map, sx, sy, gx, gy, blocked);

var path = new List<(int, int)>();
solver.PathTo(Pathfinder.Dijkstra, map, sx, sy, gx, gy, blocked, path);

var field = new int[width * height];
solver.BfsField(map, hx, hy, blocked, field);
```

`blocked` is an optional `IReadOnlyList<(int,int)>` of cells to avoid (pass `null` for none). For `AStar`/`Greedy`/`Diagonal` blocked cells are impassable; for `Dijkstra` they cost an extra 200 instead of blocking, matching the Rust danger model. Orthogonal step costs 10, diagonal 14.

## Benchmark numbers

Sample run on the 80×30 maze (28% walls), 20 000 iterations per algorithm — relative behaviour matches the Rust core:

```
algo        nodes      us/search   reached
BFS           1542       40.941   yes
A*             822       82.550   yes
Dijkstra      1543       99.947   yes
Greedy         143        9.597   yes
Diagonal      1171      133.110   yes
```

A* expands roughly half the cells BFS visits; Greedy the fewest; Dijkstra and BFS flood the whole reachable region. Absolute µs are higher than the Rust build (managed runtime, modulo/division per node) but the algorithm trade-offs are identical.
