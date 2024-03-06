import { createEffect } from "solid-js";
import * as twgl from "twgl.js";
import init, { WasmLattice } from "../snail-lattice/pkg/snail_lattice";
const m4 = twgl.m4;

import globalPosition from "./state/position";
import { SnailKey, SNAILS, SNAIL_NAMES } from "./state/shop";
import { randomSeed } from "./util";

const FRAGMENT_SHADER = `
precision mediump float;

varying vec2 v_texCoord;
uniform sampler2D u_diffuse;
uniform vec4 u_color;

void main() {
  gl_FragColor = u_color; // texture2D(u_diffuse, v_texCoord);
}
`;

const VERTEX_SHADER = `
uniform mat4 u_worldViewProjection;

attribute vec4 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;

varying vec2 v_texCoord;

void main() {
  v_texCoord = a_texcoord;
  gl_Position = (u_worldViewProjection * a_position);
}
`;


function initEngine() {
    let canvas = document.getElementById("canvas") as HTMLCanvasElement;
    const gl = canvas.getContext("webgl2", { antialias: true });
    const [globalPos, _] = globalPosition;
    twgl.setDefaults({ attribPrefix: "a_" });

    const programInfo = twgl.createProgramInfo(gl, [VERTEX_SHADER, FRAGMENT_SHADER]);
    const object = twgl.primitives.createCubeBufferInfo(gl, 2);

    const camera = m4.identity();
    const view = m4.identity();
    const viewProjection = m4.identity();

    function renderLattice(name: SnailKey, rect: DOMRect) {
        const width = (rect.right - rect.left) * devicePixelRatio;
        const height = (rect.bottom - rect.top) * devicePixelRatio;
        const left = rect.left * devicePixelRatio;
        const bottom = (canvas.clientHeight - rect.bottom) * devicePixelRatio;
        const scale = globalPos().scale;

        // if viewport is fully off screen
        if (rect.bottom < 0 || rect.top > canvas.clientHeight ||
            rect.right < 0 || rect.left > canvas.clientWidth) {
            return;
        }

        gl.viewport(left, bottom, width, height);
        gl.scissor(left, bottom, width, height);

        const mazeSize = SNAILS[name].size;
        const mazeWidth = width / (20 * scale * devicePixelRatio); // the width of the mazes in WebGL units
        const mazeHeight = height / (20 * scale * devicePixelRatio); // the height of the mazes in WebGL units

        const projection = m4.ortho(0.1, mazeWidth, -mazeHeight, -0.1, 0, 10);
        m4.multiply(projection, view, viewProjection);

        gl.useProgram(programInfo.program);

        const uni = {
            u_diffuse: tex,
            u_world: m4.identity(),
            u_worldInverseTranspose: m4.identity(),
            u_worldViewProjection: m4.identity(),
            u_color: [0.024, 0.561, 0.937, 1],
        };

        let world_initial = m4.copy(uni.u_world);
        let mazes = SNAILS[name].mazes;

        let overflow = calculateOverflow(name, rect);

        let i = 0;
        let numMazesWidth = Math.ceil(mazeWidth / (mazeSize + 0.1));
        let numMazesHeight = Math.ceil(mazeHeight / (mazeSize + 0.1));

        let maxVisibleHeight = numMazesHeight - overflow.bottom;
        let maxVisibleWidth = numMazesWidth - overflow.right;

        let visible = [];

        let [snail, _setSnail] = SNAILS[name].store;
        let mazeCount = snail.count;

        for (let y = overflow.top; y < maxVisibleHeight; y++) {
            for (let x = overflow.left; x < maxVisibleWidth; x++) {
                i = y * numMazesWidth + x;
                if (i >= mazeCount) continue;
                visible.push(i);
            }
        }

        updateLatticeMazes(name, visible);

        // render mazes
        for (let i of visible) {
            let x = i % numMazesWidth;
            let y = Math.floor(i / numMazesWidth);

            m4.translate(world_initial, [x * (mazeSize + 0.1), -y * (mazeSize + 0.1), 0], uni.u_world);
            m4.transpose(m4.inverse(uni.u_world, uni.u_worldInverseTranspose), uni.u_worldInverseTranspose);
            m4.multiply(viewProjection, uni.u_world, uni.u_worldViewProjection);

            twgl.setBuffersAndAttributes(gl, programInfo, mazes[i]);
            twgl.setUniforms(programInfo, uni);
            twgl.drawBufferInfo(gl, mazes[i]);
        }

        // get all of the snails
        // let snails = SNAIL[name].render(rendered);
    }

    function calculateOverflow(name: SnailKey, rect: DOMRect) {
        // size of a maze in pixels
        let mazeSize = SNAILS[name].size * 20 * globalPos().scale;

        let leftOverflow = Math.floor(Math.max(-rect.left / mazeSize, 0));
        let rightOverflow = Math.floor(Math.max((rect.right - window.innerWidth) / mazeSize, 0));
        let topOverflow = Math.floor(Math.max(-rect.top / mazeSize, 0));
        let bottomOverflow = Math.floor(Math.max((rect.bottom - window.innerHeight) / mazeSize, 0));

        return { left: leftOverflow, right: rightOverflow, top: topOverflow, bottom: bottomOverflow };
    }

    // TODO: make async
    function updateLatticeMazes(name: SnailKey, visible: number[]) {
        const meshes = SNAILS[name].lattice.get_meshes(visible);

        for (let mesh of meshes) {
            SNAILS[name].mazes[mesh.id] = twgl.createBufferInfoFromArrays(gl, {
                position: { numComponents: 2, data: mesh.vertices },
                indices: { numComponents: 3, data: mesh.indices },
            });
        }
    }

    // initialize all lattices
    for (let name of SNAIL_NAMES) {
        let [snail, _setSnail] = SNAILS[name].store;
        SNAILS[name].lattice = new WasmLattice(name, randomSeed());

        createEffect((prevCount: number) => {
            let count = snail.count;
            let diff = count - prevCount;

            SNAILS[name].lattice.alter(diff);

            return count;
        }, 0);
    }

    const tex = twgl.createTexture(gl, {
        min: gl.NEAREST,
        mag: gl.NEAREST,
        src: [
            255, 255, 255, 255,
            192, 192, 192, 255,
            192, 192, 192, 255,
            255, 255, 255, 255,
        ],
    });

    const render = (time: number) => {
        time *= 0.001;
        twgl.resizeCanvasToDisplaySize(canvas, window.devicePixelRatio);

        gl.enable(gl.DEPTH_TEST);
        gl.disable(gl.SCISSOR_TEST);
        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        gl.enable(gl.SCISSOR_TEST);
        gl.clearColor(0, 0, 0, 0);

        const eye = [0, 0, 8];
        const target = [0, 0, 0];
        const up = [0, 1, 0];

        m4.lookAt(eye, target, up, camera);
        m4.inverse(camera, view);

        // view elements
        let elements = Array.from(document.querySelectorAll(".positioned:has(.viewport)")) as HTMLElement[];
        elements.sort((a, b) => +a.style["z-index"] - +b.style["z-index"]);

        const scale = globalPos().scale;

        // loop
        for (let element of elements) {
            // first we clear based on the outline
            let rect = element.getBoundingClientRect();
            let width = (rect.right - rect.left - 16 * scale) * devicePixelRatio;
            let height = (rect.bottom - rect.top - 16 * scale) * devicePixelRatio;
            let left = (rect.left + 8 * scale) * devicePixelRatio;
            let bottom = (canvas.clientHeight - rect.bottom + 8 * scale) * devicePixelRatio;

            gl.viewport(left, bottom, width, height);
            gl.scissor(left, bottom, width, height);
            gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

            // render the actual content
            const viewElement = element.querySelector(".viewport");
            rect = viewElement.getBoundingClientRect();

            const mazeName = viewElement.id as SnailKey;
            renderLattice(mazeName, rect);
        }
    }

    const renderContinuously = function(time: number) {
        render(time);
        requestAnimationFrame(renderContinuously);
    };
    requestAnimationFrame(renderContinuously);
}

export function start() {
    init().then(() => {
        initEngine();
    });
}
