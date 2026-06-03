using System.Collections.Generic;
using System.Diagnostics;
using System.Text;

namespace Abyssal.Pathfinding
{
    public struct BenchResult
    {
        public Pathfinder Pf;
        public int Nodes;
        public double MicrosPerSearch;
        public bool Reached;
    }

    public static class PathfindingBenchmark
    {
        public static GridMap BuildMaze(int width, int height, int wallPercent, uint seed)
        {
            var walk = new bool[width * height];
            uint state = seed == 0 ? 1u : seed;
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    bool border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    bool wall = border || (state % 100) < (uint)wallPercent;
                    walk[y * width + x] = !wall;
                }
            }
            walk[1 * width + 1] = true;
            walk[(height - 2) * width + (width - 2)] = true;
            return Reachable(walk, width, height) ? new GridMap(width, height, walk) : BuildMaze(width, height, wallPercent > 4 ? wallPercent - 2 : wallPercent, seed + 1);
        }

        private static bool Reachable(bool[] walk, int w, int h)
        {
            var map = new GridMap(w, h, walk);
            var solver = new PathSolver();
            var dist = new int[w * h];
            solver.BfsField(map, 1, 1, null, dist);
            return dist[(h - 2) * w + (w - 2)] >= 0;
        }

        public static List<BenchResult> Run(int width, int height, int iterations, uint seed)
        {
            var map = BuildMaze(width, height, 28, seed);
            int sx = 1, sy = 1;
            int gx = width - 2, gy = height - 2;
            var solver = new PathSolver();
            var results = new List<BenchResult>(PathfinderInfo.All.Length);

            foreach (var pf in PathfinderInfo.All)
            {
                for (int w = 0; w < 64; w++) solver.StepTo(pf, map, sx, sy, gx, gy, null);
                int nodes = solver.SearchCost(pf, map, sx, sy, gx, gy, null);
                bool reached = solver.StepTo(pf, map, sx, sy, gx, gy, null).HasValue;

                var sw = Stopwatch.StartNew();
                for (int i = 0; i < iterations; i++)
                {
                    solver.StepTo(pf, map, sx, sy, gx, gy, null);
                }
                sw.Stop();
                double micros = sw.Elapsed.TotalMilliseconds * 1000.0 / iterations;

                results.Add(new BenchResult { Pf = pf, Nodes = nodes, MicrosPerSearch = micros, Reached = reached });
            }
            return results;
        }

        public static string Format(List<BenchResult> results, int width, int height, int iterations)
        {
            var sb = new StringBuilder();
            sb.AppendLine($"Pathfinding benchmark  {width}x{height} grid  {iterations} iterations/algo");
            sb.AppendLine("algo        nodes      us/search   reached");
            foreach (var r in results)
            {
                sb.AppendLine($"{r.Pf.Label(),-10}  {r.Nodes,6}     {r.MicrosPerSearch,8:F3}   {(r.Reached ? "yes" : "no")}");
            }
            return sb.ToString();
        }
    }
}
