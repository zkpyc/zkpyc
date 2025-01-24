from zk_types.types import Private, Array # zk_ignore

N: int = 9
M: int = 3

sudoku_puzzle: Array[Array[int, 9], 9] = [
    [5, 3, 0, 0, 7, 0, 0, 0, 0],
    [6, 0, 0, 1, 9, 5, 0, 0, 0],
    [0, 9, 8, 0, 0, 0, 0, 6, 0],
    [8, 0, 0, 0, 6, 0, 0, 0, 3],
    [4, 0, 0, 8, 0, 3, 0, 0, 1],
    [7, 0, 0, 0, 2, 0, 0, 0, 6],
    [0, 6, 0, 0, 0, 0, 2, 8, 0],
    [0, 0, 0, 4, 1, 9, 0, 0, 5],
    [0, 0, 0, 0, 8, 0, 0, 7, 9],
]

def sudoku_constraints(grid: Array[Array[int, 9], 9]) -> bool:
    # Check that cell value is in correct range
    for i in range(N):
        for j in range(N):
            assert 1 <= grid[i][j] <= 9, "Each cell must contain a number between 1 and 9."

    # Check that rows have unique values
    for i in range(N):
        for j in range(N):
            for k in range(j + 1, N):
                assert grid[i][j] != grid[i][k], "Each row must contain unique values."

    # Check that columns have unique values
    for j in range(N):
        for i in range(N):
            for k in range(i + 1, N):
                assert grid[i][j] != grid[k][j], "Each column must contain unique values."

    # Check that each 3x3 sub-grid has unique values
    for x in range(N // M):
        for y in range(N // M):
            subgrid: Array[int, 9] = [0 for _ in range(9)]
            idx: int = 0
            for i in range(M):
                for j in range(M):
                    subgrid[idx] = grid[x * M + i][y * M + j]
                    idx += 1
            for i in range(M):
                for j in range(i + 1, M):
                    assert subgrid[i] != subgrid[j], "Each 3x3 sub-grid must contain unique values."
    
    return True

def main(witness: Private[Array[int, 81]]) -> bool:
    grid: Array[Array[int, 9], 9] = [[0 for _ in range(N)] for _ in range(N)]
    for i in range(N):
        for j in range(N):
            grid[i][j] = witness[i * N + j]
    
    # Check the solution is valid
    for i in range(N):
        for j in range(N):
            assert grid[i][j] == sudoku_puzzle[i][j] or sudoku_puzzle[i][j] == 0, "Provided solution does not match the original puzzle."

    return True if sudoku_constraints(grid) == True else False
