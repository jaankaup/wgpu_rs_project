# Fast marching method

The algorithm description.

## Structs

struct FMM_Block {
    index: u32,
    band_points: u32,
}

struct FMM_Node {
    value: f32,
    tag: u32,
};

enum tag {
    0, // Far
    1, // Band
    2, // Known
}

## Grid (2d case)

|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
|   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |
| 12| 13| 14| 15|   |   |   |   |   |   |   |   |   |   |   |   |
| 8 | 9 | 10| 11|   |   |   |   |   |   |   |   |   |   |   |   |
| 4 | 5 | 6 | 7 |   |   |   |   |   |   |   |   |   |   |   |   |
| 0 | 1 | 2 | 3 |   |   |   |   |   |   |   |   |   |   |   |   |


## The initializtion phase

- Create a uniform grid (the fmm domain) and mark each grid as far cell.
- Define speed information.
- Define the initial boundary (known points).
- Calculate the initial band points. 

## The loop.

0. Scan all FMM_Blocks and collect all FMM_Blocks that has atleast one band
   point (prefix sum and stream compaction).
1. Break the loop if the aren't any active FMM_Blocks (break condition).
2. For each sub-domain that contains band points, load all cell points to shared memory.
   Load the ghost zone data (the overlapping neighbor data) to shared memory.
3. On each sub-domain, find the smallest band cell using reduction and change its tag to known.
4. Update the surrounding neighbor cells from far to band and perform fmm on them (scan/compaction first?).
5. Synchronize ghost domain data.
