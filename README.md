# Tulya–MDL: Deterministic MDL-Based Block Alignment

A deterministic algorithm that aligns two strings by minimizing description length.

## Key Idea

* **Blocks**: Shared structure identified by the algorithm to minimize descriptive cost.
* **Uncovered**: Residual parts of the strings that are not part of any block.
* **Cost Minimization**: The algorithm iteratively minimizes the total encoding cost (description length) by selecting the most efficient block configurations.

## Example

Command:
```bash
cargo run -- -s abcxyz -t abwxy -v v1 --debug
```

Trimmed Output:
```text
[Step 1]
Energy: 89 → 61
[Step 2]
Energy: 61 → 38

╔══════════════════════════════════════════════════════════════════╗
║                         BLOCK TABLE                              ║
╚══════════════════════════════════════════════════════════════════╝
+===========================================================================+
| ID | Len | Source Range | Target Range | Density | Fragments | Content |
+===========================================================================+
| 0  | 2   | 0..1         | 0..1         | 1.00    | 1         | ab      |
|----+-----+--------------+--------------+---------+-----------+---------|
| 1  | 2   | 3..4         | 3..4         | 1.00    | 1         | xy      |
+----+-----+--------------+--------------+---------+-----------+---------+

╔══════════════════════════════════════════════════════════════════╗
║                      ASCII ALIGNMENT VIEW                        ║
╚══════════════════════════════════════════════════════════════════╝
Source: ab c x y z
Target: ab w x y

Final Energy: 38
```

## Features

* **Deterministic local search**: Always produces consistent results for the same input.
* **Guaranteed convergence**: The algorithm is guaranteed to reach a local optimum.
* **Interpretable block structure**: Provides clear insights into shared patterns.
* **CLI visualization**: Includes detailed alignment table, energy breakdown, and segments view.

## How to Run

```bash
cargo build
cargo run -- -s abcxyz -t abwxy --debug
```

## Paper

Link to: [paper/tulya_mdl.pdf](paper/tulya_mdl.pdf)
