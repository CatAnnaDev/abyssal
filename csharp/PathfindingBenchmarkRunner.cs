#if UNITY_2020_1_OR_NEWER
using UnityEngine;

namespace Abyssal.Pathfinding
{
    public sealed class PathfindingBenchmarkRunner : MonoBehaviour
    {
        public int width = 80;
        public int height = 30;
        public int iterations = 20000;
        public int seed = 1337;
        public bool runOnStart = true;

        private void Start()
        {
            if (runOnStart) RunBenchmark();
        }

        [ContextMenu("Run Benchmark")]
        public void RunBenchmark()
        {
            var results = PathfindingBenchmark.Run(width, height, iterations, (uint)seed);
            Debug.Log(PathfindingBenchmark.Format(results, width, height, iterations));
        }

        [ContextMenu("Demo Path")]
        public void DemoPath()
        {
            var map = PathfindingBenchmark.BuildMaze(width, height, 28, (uint)seed);
            var solver = new PathSolver();
            var path = new System.Collections.Generic.List<(int, int)>();
            bool found = solver.PathTo(Pathfinder.AStar, map, 1, 1, width - 2, height - 2, null, path);
            Debug.Log(found ? $"A* path length {path.Count}, expanded {solver.LastNodes} nodes" : "no path");
        }
    }
}
#endif
