using System.Collections.Generic;

namespace Abyssal.Pathfinding
{
    public enum Pathfinder
    {
        Bfs,
        AStar,
        Dijkstra,
        Greedy,
        Diagonal,
    }

    public static class PathfinderInfo
    {
        public static readonly Pathfinder[] All =
        {
            Pathfinder.Bfs,
            Pathfinder.AStar,
            Pathfinder.Dijkstra,
            Pathfinder.Greedy,
            Pathfinder.Diagonal,
        };

        public static string Label(this Pathfinder pf)
        {
            switch (pf)
            {
                case Pathfinder.Bfs: return "BFS";
                case Pathfinder.AStar: return "A*";
                case Pathfinder.Dijkstra: return "Dijkstra";
                case Pathfinder.Greedy: return "Greedy";
                default: return "Diagonal";
            }
        }
    }

    public sealed class GridMap
    {
        public readonly int Width;
        public readonly int Height;
        private readonly bool[] _walk;

        public GridMap(int width, int height, bool[] walkable)
        {
            Width = width;
            Height = height;
            _walk = walkable;
        }

        public int Idx(int x, int y) => y * Width + x;
        public bool InBounds(int x, int y) => x >= 0 && y >= 0 && x < Width && y < Height;
        public bool Walkable(int index) => _walk[index];
    }

    public sealed class PathSolver
    {
        private static readonly int[] DX4 = { 0, 0, -1, 1 };
        private static readonly int[] DY4 = { -1, 1, 0, 0 };
        private static readonly int[] DX8 = { 0, 0, -1, 1, -1, 1, -1, 1 };
        private static readonly int[] DY8 = { -1, 1, 0, 0, -1, -1, 1, 1 };

        private int _len;
        private uint _vgen;
        private uint _bgen;
        private uint[] _seen = System.Array.Empty<uint>();
        private uint[] _closed = System.Array.Empty<uint>();
        private int[] _g = System.Array.Empty<int>();
        private int[] _came = System.Array.Empty<int>();
        private uint[] _block = System.Array.Empty<uint>();

        private long[] _heap = new long[64];
        private int _heapCount;
        private int[] _queue = new int[64];
        private int _qHead;
        private int _qTail;

        public long LastNodes { get; private set; }

        private void Begin(int n, IReadOnlyList<(int, int)> blocked, int w)
        {
            if (_len < n)
            {
                System.Array.Resize(ref _seen, n);
                System.Array.Resize(ref _closed, n);
                System.Array.Resize(ref _g, n);
                System.Array.Resize(ref _came, n);
                System.Array.Resize(ref _block, n);
                if (_queue.Length < n + 1)
                {
                    _queue = new int[n + 1];
                }
                _len = n;
            }
            unchecked { _vgen++; }
            if (_vgen == 0)
            {
                System.Array.Clear(_seen, 0, _seen.Length);
                System.Array.Clear(_closed, 0, _closed.Length);
                _vgen = 1;
            }
            unchecked { _bgen++; }
            if (_bgen == 0)
            {
                System.Array.Clear(_block, 0, _block.Length);
                _bgen = 1;
            }
            if (blocked != null)
            {
                for (int k = 0; k < blocked.Count; k++)
                {
                    var (bx, by) = blocked[k];
                    if (bx >= 0 && by >= 0)
                    {
                        int i = by * w + bx;
                        if (i < n) _block[i] = _bgen;
                    }
                }
            }
            _heapCount = 0;
            _qHead = 0;
            _qTail = 0;
        }

        private bool IsBlock(int i) => _block[i] == _bgen;
        private int GVal(int i) => _seen[i] == _vgen ? _g[i] : int.MaxValue;

        private void Relax(int i, int g, int parent)
        {
            _seen[i] = _vgen;
            _g[i] = g;
            _came[i] = parent;
        }

        private void HeapPush(long key)
        {
            if (_heapCount == _heap.Length) System.Array.Resize(ref _heap, _heap.Length * 2);
            int c = _heapCount++;
            _heap[c] = key;
            while (c > 0)
            {
                int p = (c - 1) >> 1;
                if (_heap[p] <= _heap[c]) break;
                (_heap[p], _heap[c]) = (_heap[c], _heap[p]);
                c = p;
            }
        }

        private long HeapPop()
        {
            long top = _heap[0];
            _heapCount--;
            _heap[0] = _heap[_heapCount];
            int c = 0;
            while (true)
            {
                int l = c * 2 + 1;
                int r = l + 1;
                int small = c;
                if (l < _heapCount && _heap[l] < _heap[small]) small = l;
                if (r < _heapCount && _heap[r] < _heap[small]) small = r;
                if (small == c) break;
                (_heap[small], _heap[c]) = (_heap[c], _heap[small]);
                c = small;
            }
            return top;
        }

        private static long Pack(int priority, int index) => ((long)priority << 32) | (uint)index;

        public (int, int)? StepTo(Pathfinder pf, GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked)
        {
            if (pf == Pathfinder.Bfs) return BfsStep(map, sx, sy, gx, gy, blocked);
            Flags(pf, out bool useG, out bool useH, out bool diag, out bool weighted, out bool hardBlock);
            int goal = BestFirst(map, sx, sy, gx, gy, blocked, useG, useH, diag, weighted, hardBlock);
            return goal < 0 ? null : FirstStep(map.Width, map.Idx(sx, sy), goal, sx, sy);
        }

        public int SearchCost(Pathfinder pf, GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked)
        {
            if (pf == Pathfinder.Bfs)
            {
                BfsStep(map, sx, sy, gx, gy, blocked);
                return (int)LastNodes;
            }
            Flags(pf, out bool useG, out bool useH, out bool diag, out bool weighted, out bool hardBlock);
            BestFirst(map, sx, sy, gx, gy, blocked, useG, useH, diag, weighted, hardBlock);
            return (int)LastNodes;
        }

        public bool PathTo(Pathfinder pf, GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked, List<(int, int)> outPath)
        {
            outPath.Clear();
            int goal;
            int start = map.Idx(sx, sy);
            if (pf == Pathfinder.Bfs)
            {
                goal = BfsGoal(map, sx, sy, gx, gy, blocked);
            }
            else
            {
                Flags(pf, out bool useG, out bool useH, out bool diag, out bool weighted, out bool hardBlock);
                goal = BestFirst(map, sx, sy, gx, gy, blocked, useG, useH, diag, weighted, hardBlock);
            }
            if (goal < 0) return false;
            int cur = goal;
            while (cur != start)
            {
                outPath.Add((cur % map.Width, cur / map.Width));
                cur = _came[cur];
            }
            outPath.Reverse();
            return true;
        }

        public void BfsField(GridMap map, int sx, int sy, IReadOnlyList<(int, int)> blocked, int[] dist)
        {
            int w = map.Width;
            int n = w * map.Height;
            for (int i = 0; i < n; i++) dist[i] = -1;
            if (!map.InBounds(sx, sy)) return;
            Begin(n, blocked, w);
            int start = sy * w + sx;
            dist[start] = 0;
            QPush(start);
            while (_qHead != _qTail)
            {
                int ci = QPop();
                int cx = ci % w;
                int cy = ci / w;
                int d = dist[ci];
                for (int k = 0; k < 4; k++)
                {
                    int nx = cx + DX4[k];
                    int ny = cy + DY4[k];
                    if (!map.InBounds(nx, ny)) continue;
                    int ni = ny * w + nx;
                    if (dist[ni] >= 0 || !map.Walkable(ni) || IsBlock(ni)) continue;
                    dist[ni] = d + 1;
                    QPush(ni);
                }
            }
        }

        private static void Flags(Pathfinder pf, out bool useG, out bool useH, out bool diag, out bool weighted, out bool hardBlock)
        {
            switch (pf)
            {
                case Pathfinder.AStar: useG = true; useH = true; diag = false; weighted = false; hardBlock = true; break;
                case Pathfinder.Greedy: useG = false; useH = true; diag = false; weighted = false; hardBlock = true; break;
                case Pathfinder.Dijkstra: useG = true; useH = false; diag = false; weighted = true; hardBlock = false; break;
                case Pathfinder.Diagonal: useG = true; useH = true; diag = true; weighted = false; hardBlock = true; break;
                default: useG = true; useH = false; diag = false; weighted = false; hardBlock = true; break;
            }
        }

        private void QPush(int v)
        {
            _queue[_qTail] = v;
            _qTail++;
            if (_qTail == _queue.Length) _qTail = 0;
        }

        private int QPop()
        {
            int v = _queue[_qHead];
            _qHead++;
            if (_qHead == _queue.Length) _qHead = 0;
            return v;
        }

        private (int, int)? BfsStep(GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked)
        {
            int goal = BfsGoal(map, sx, sy, gx, gy, blocked);
            return goal < 0 ? null : FirstStep(map.Width, map.Idx(sx, sy), goal, sx, sy);
        }

        private int BfsGoal(GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked)
        {
            LastNodes = 0;
            if ((sx == gx && sy == gy) || !map.InBounds(sx, sy)) return -1;
            int w = map.Width;
            int n = w * map.Height;
            Begin(n, blocked, w);
            int start = sy * w + sx;
            _seen[start] = _vgen;
            _came[start] = start;
            QPush(start);
            while (_qHead != _qTail)
            {
                int ci = QPop();
                LastNodes++;
                int cx = ci % w;
                int cy = ci / w;
                if (cx == gx && cy == gy) return ci;
                for (int k = 0; k < 4; k++)
                {
                    int nx = cx + DX4[k];
                    int ny = cy + DY4[k];
                    if (!map.InBounds(nx, ny)) continue;
                    int ni = ny * w + nx;
                    if (_seen[ni] == _vgen) continue;
                    bool goal = nx == gx && ny == gy;
                    if (!goal && (!map.Walkable(ni) || IsBlock(ni))) continue;
                    _seen[ni] = _vgen;
                    _came[ni] = ci;
                    if (goal) return ni;
                    QPush(ni);
                }
            }
            return -1;
        }

        private int BestFirst(GridMap map, int sx, int sy, int gx, int gy, IReadOnlyList<(int, int)> blocked, bool useG, bool useH, bool diag, bool weighted, bool hardBlock)
        {
            LastNodes = 0;
            if ((sx == gx && sy == gy) || !map.InBounds(sx, sy)) return -1;
            int w = map.Width;
            int n = w * map.Height;
            Begin(n, blocked, w);
            int start = sy * w + sx;
            Relax(start, 0, start);
            HeapPush(Pack(0, start));
            int dirs = diag ? 8 : 4;
            int[] dxs = diag ? DX8 : DX4;
            int[] dys = diag ? DY8 : DY4;
            while (_heapCount > 0)
            {
                int ci = (int)(uint)HeapPop();
                if (_closed[ci] == _vgen) continue;
                _closed[ci] = _vgen;
                LastNodes++;
                int cx = ci % w;
                int cy = ci / w;
                if (cx == gx && cy == gy) return ci;
                int cg = _g[ci];
                for (int k = 0; k < dirs; k++)
                {
                    int dx = dxs[k];
                    int dy = dys[k];
                    int nx = cx + dx;
                    int ny = cy + dy;
                    if (!map.InBounds(nx, ny)) continue;
                    bool goal = nx == gx && ny == gy;
                    int ni = ny * w + nx;
                    if (!goal && !map.Walkable(ni)) continue;
                    bool costly = IsBlock(ni);
                    if (costly && !goal && hardBlock) continue;
                    int baseCost = (dx != 0 && dy != 0) ? 14 : 10;
                    int extra = (weighted && costly) ? 200 : 0;
                    int ng = cg + baseCost + extra;
                    if (ng < GVal(ni))
                    {
                        Relax(ni, ng, ci);
                        int hh = 0;
                        if (useH)
                        {
                            int adx = nx > gx ? nx - gx : gx - nx;
                            int ady = ny > gy ? ny - gy : gy - ny;
                            hh = (diag ? (adx > ady ? adx : ady) : adx + ady) * 10;
                        }
                        int pri = (useG ? ng : 0) + hh;
                        HeapPush(Pack(pri, ni));
                    }
                }
            }
            return -1;
        }

        private (int, int) FirstStep(int w, int start, int goal, int sx, int sy)
        {
            int cur = goal;
            while (_came[cur] != start && _came[cur] != -1)
            {
                cur = _came[cur];
            }
            return (cur % w - sx, cur / w - sy);
        }
    }
}
