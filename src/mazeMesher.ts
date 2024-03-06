enum WallDir {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

enum CellState {
    WALL,
    EMPTY,
}

export class Maze {
    data: Uint8Array;
    width: number;
    height: number;

    constructor(width: number, height: number) {
        // 0 === wall
        this.data = new Uint8Array((2 * height + 1) * (2 * width + 1));
        this.width = width;
        this.height = height;

        this.mazeVisit(0, 0);

        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                if (this.getTile(x, y) === CellState.WALL) {
                    this.joinWall(x, y);
                    this.mazeVisit(x, y);
                }
            }
        }
    }

    joinWall(x: number, y: number) {
        for (let dir of [WallDir.UP, WallDir.DOWN, WallDir.LEFT, WallDir.RIGHT]) {
            switch (dir) {
                case WallDir.UP:
                    if (y > 0 && this.getTile(x, y - 1) === CellState.EMPTY) {
                        this.setTileWall(x, y, WallDir.UP, CellState.EMPTY);
                        return;
                    }
                    break;
                case WallDir.DOWN:
                    if (y < this.height - 1 && this.getTile(x, y + 1) === CellState.EMPTY) {
                        this.setTileWall(x, y, WallDir.DOWN, CellState.EMPTY);
                        return;
                    }
                    break;
                case WallDir.LEFT:
                    if (x > 0 && this.getTile(x - 1, y) === CellState.EMPTY) {
                        this.setTileWall(x, y, WallDir.LEFT, CellState.EMPTY);
                        return;
                    }
                    break;
                case WallDir.RIGHT:
                    if (x < this.width - 1 && this.getTile(x + 1, y) === CellState.EMPTY) {
                        this.setTileWall(x, y, WallDir.RIGHT, CellState.EMPTY);
                        return;
                    }
                    break;
            }
        }
    }

    mazeVisit(x: number, y: number) {
        this.setTile(x, y, CellState.EMPTY);
        let dir = Math.round(Math.random() * 4);

        switch (dir) {
            case WallDir.UP:
                if (y > 0 && this.getTile(x, y - 1) === CellState.WALL) {
                    this.setTileWall(x, y, WallDir.UP, CellState.EMPTY);
                    this.mazeVisit(x, y - 1);
                }

                return;
            case WallDir.DOWN:
                if (y < this.height - 1 && this.getTile(x, y + 1) === CellState.WALL) {
                    this.setTileWall(x, y, WallDir.DOWN, CellState.EMPTY);
                    this.mazeVisit(x, y + 1);
                }

                return;
            case WallDir.LEFT:
                if (x > 0 && this.getTile(x - 1, y) === CellState.WALL) {
                    this.setTileWall(x, y, WallDir.LEFT, CellState.EMPTY);
                    this.mazeVisit(x - 1, y);
                }

                return;
            case WallDir.RIGHT:
                if (x < this.width - 1 && this.getTile(x + 1, y) === CellState.WALL) {
                    this.setTileWall(x, y, WallDir.RIGHT, CellState.EMPTY);
                    this.mazeVisit(x + 1, y);
                }

                return;
        }
    }

    debugPrint() {
        let out = "";

        let w = this.width * 2 + 1;

        for (let y = 0; y < this.height * 2 + 1; y++) {
            for (let x = 0; x < this.width * 2 + 1; x++) {
                out += this.data[y * w + x] === CellState.WALL ? "#" : " ";
            }
            out += "\n";
        }

        console.log(out);
    }

    setTile(x: number, y: number, state: CellState) {
        this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2 + 1)] = state;
    }

    getTile(x: number, y: number): CellState {
        return this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2 + 1)];
    }

    setTileWall(x: number, y: number, dir: WallDir, state: CellState) {
        switch (dir) {
            case WallDir.UP:
                this.data[(y * 2) * (2 * this.width + 1) + (x * 2 + 1)] = state;
                break;
            case WallDir.DOWN:
                this.data[(y * 2 + 2) * (2 * this.width + 1) + (x * 2 + 1)] = state;
                break;
            case WallDir.LEFT:
                this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2)] = state;
                break;
            case WallDir.RIGHT:
                this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2 + 2)] = state;
                break;
        }
    }

    getTileWall(x: number, y: number, dir: WallDir): CellState {
        switch (dir) {
            case WallDir.UP:
                return this.data[(y * 2) * (2 * this.width + 1) + (x * 2 + 1)];
            case WallDir.DOWN:
                return this.data[(y * 2 + 2) * (2 * this.width + 1) + (x * 2 + 1)];
            case WallDir.LEFT:
                return this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2)];
            case WallDir.RIGHT:
                return this.data[(y * 2 + 1) * (2 * this.width + 1) + (x * 2 + 2)];
        }
    }

    // currently inefficient in terms of the generated geometry
    meshify(): [Float32Array, Uint16Array] {
        let vertices = new Float32Array(2 * 4 * (this.width + 1) * (this.height + 1));
        let indices = new Uint16Array(3 * 4 * (this.width + 1) * (this.height + 1));

        // i represents the number of vertices
        let i = 0;
        let ii = 0;
        for (let y = 0; y <= this.height; y++) {
            for (let x = 0; x <= this.width; x++) {
                // top left
                vertices[i * 2 + 0] = (0.0 + x);
                vertices[i * 2 + 1] = -(0.0 + y);

                // top right
                vertices[i * 2 + 2] = (0.1 + x);
                vertices[i * 2 + 3] = -(0.0 + y);

                // bottom left
                vertices[i * 2 + 4] = (0.0 + x);
                vertices[i * 2 + 5] = -(0.1 + y);

                // bottom right
                vertices[i * 2 + 6] = (0.1 + x);
                vertices[i * 2 + 7] = -(0.1 + y);

                indices[ii++] = i; // TL
                indices[ii++] = i + 1; // TR
                indices[ii++] = i + 2; // BL

                indices[ii++] = i + 2; // BL
                indices[ii++] = i + 3; // BR
                indices[ii++] = i + 1; // TR

                i += 4;
            }
        }

        i = 0;
        for (let y = 0; y < this.height; y++) {
            for (let x = 0; x < this.width; x++) {
                if (this.getTileWall(x, y, WallDir.UP) === CellState.WALL) {
                    indices[ii++] = i + 1; // left-TR
                    indices[ii++] = i + 4 + 0; // right-TL
                    indices[ii++] = i + 3; // left-BR

                    indices[ii++] = i + 3; // left-BR
                    indices[ii++] = i + 4 + 0; // right-TL
                    indices[ii++] = i + 4 + 2; // right-BL
                }

                if (this.getTileWall(x, y, WallDir.LEFT) === CellState.WALL) {
                    indices[ii++] = i + 2; // top-BL
                    indices[ii++] = i + 3; // top-BR
                    indices[ii++] = i + 4 * (this.width + 1) + 0; // bottom-TL

                    indices[ii++] = i + 4 * (this.width + 1) + 0; // bottom-TL
                    indices[ii++] = i + 4 * (this.width + 1) + 1; // bottom-TR
                    indices[ii++] = i + 3; // top-BR
                }

                i += 4;
            }

            indices[ii++] = i + 2; // top-BL
            indices[ii++] = i + 3; // top-BR
            indices[ii++] = i + 4 * (this.width + 1) + 0; // bottom-TL

            indices[ii++] = i + 4 * (this.width + 1) + 0; // bottom-TL
            indices[ii++] = i + 4 * (this.width + 1) + 1; // bottom-TR
            indices[ii++] = i + 3; // top-BR
            i += 4;
        }

        for (let x = 0; x < this.width; x++) {
            indices[ii++] = i + 1; // left-TR
            indices[ii++] = i + 4 + 0; // right-TL
            indices[ii++] = i + 3; // left-BR

            indices[ii++] = i + 3; // left-BR
            indices[ii++] = i + 4 + 0; // right-TL
            indices[ii++] = i + 4 + 2; // right-BL

            i += 4;
        }

        return [vertices, indices];
    }
}
