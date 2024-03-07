import { createEffect } from "solid-js";
import * as twgl from "twgl.js";
import init, { WasmLattice } from "../snail-lattice/pkg/snail_lattice";
const m4 = twgl.m4;

import globalPosition from "./state/position";
import { SnailKey, SNAILS, SNAIL_NAMES } from "./state/shop";
import { randomSeed } from "./util";

const QUAD_VERTICES = [
    0, 0,
    0, 0.8,
    0.8, 0,
    0.8, 0.8
];

const FRAGMENT_SHADER = `
precision mediump float;

varying vec2 v_texCoord;
uniform sampler2D u_diffuse;

void main() {
  gl_FragColor = texture2D(u_diffuse, v_texCoord);
}
`;

const VERTEX_SHADER = `
uniform mat4 u_worldViewProjection;

attribute vec4 a_position;
attribute vec2 a_instance_pos;
attribute vec2 a_texcoord;

varying vec2 v_texCoord;

void main() {
  v_texCoord = a_texcoord;
  gl_Position = (u_worldViewProjection * vec4((a_position.xy + a_instance_pos), 0, 1));
}
`;


function initEngine() {
    let canvas = document.getElementById("canvas") as HTMLCanvasElement;
    const gl = canvas.getContext("webgl2", { antialias: true, alpha: true });

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const [globalPos, _] = globalPosition;
    twgl.setDefaults({ attribPrefix: "a_" });

    const programInfo = twgl.createProgramInfo(gl, [VERTEX_SHADER, FRAGMENT_SHADER]);
    const object = twgl.primitives.createCubeBufferInfo(gl, 2);

    const camera = m4.identity();
    const view = m4.identity();
    const viewProjection = m4.identity();

    function renderLattice(name: SnailKey, rect: DOMRect) {
        const adjLeft = Math.max(rect.left, 0);
        const adjRight = Math.min(rect.right, canvas.clientWidth);
        const adjTop = Math.max(rect.top, 0);
        const adjBottom = Math.min(rect.bottom, canvas.clientHeight);

        const leftOffset = rect.left - adjLeft;
        const topOffset = rect.top - adjTop;

        const width = (adjRight - adjLeft) * devicePixelRatio;
        const height = (adjBottom - adjTop) * devicePixelRatio;
        const left = adjLeft * devicePixelRatio;
        const bottom = (canvas.clientHeight - adjBottom) * devicePixelRatio;
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
        };

        let world_initial = m4.copy(uni.u_world);
        m4.translate(world_initial, [leftOffset / (20 * scale), -topOffset / (20 * scale), 0], world_initial);

        let mazes = SNAILS[name].mazes;

        let overflow = calculateOverflow(name, rect);

        let [snail, _setSnail] = SNAILS[name].store;
        let i = 0;
        let numMazesWidth = snail.width;
        let numMazesHeight = snail.height;

        let maxVisibleHeight = numMazesHeight - overflow.bottom;
        let maxVisibleWidth = numMazesWidth - overflow.right;

        let visible = [];

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

        m4.translate(world_initial, [0, 0, 1], uni.u_world);
        m4.transpose(m4.inverse(uni.u_world, uni.u_worldInverseTranspose), uni.u_worldInverseTranspose);
        m4.multiply(viewProjection, uni.u_world, uni.u_worldViewProjection);
        uni.u_diffuse = textures["snail"];
        twgl.setUniforms(programInfo, uni);

        // gl.clear(gl.DEPTH_BUFFER_BIT); // TODO: probably not ideal, but i don't care for now

        // get all of the snails
        // @ts-ignore
        let snails = SNAILS[name].lattice.render(visible);

        const quadPositions = new Float32Array(visible.length * 2);
        for (let i = 0; i < visible.length; i++) {
            let x = visible[i] % numMazesWidth;
            let y = Math.floor(visible[i] / numMazesWidth);

            quadPositions[i * 2] = x * (mazeSize + 0.1) + 0.1 + snails[i * 3 + 1] / 10;
            quadPositions[i * 2 + 1] = -y * (mazeSize + 0.1) - 1 - snails[i * 3 + 2] / 10;
        }


        const quadTexCoords = [
            0.0, 1,
            0.0, 0.0,
            8 / 18, 1,
            8 / 18, 0.0,
        ];

        let snailBufferInfo = twgl.createBufferInfoFromArrays(gl, {
            position: { numComponents: 2, data: QUAD_VERTICES },
            instance_pos: { numComponents: 2, data: quadPositions, divisor: 1 },
            texcoord: { numComponents: 2, data: quadTexCoords }
        });

        twgl.setBuffersAndAttributes(gl, programInfo, snailBufferInfo);
        twgl.drawBufferInfo(gl, snailBufferInfo, gl.TRIANGLE_STRIP, snailBufferInfo.numElements, 0, visible.length);
    }

    function calculateOverflow(name: SnailKey, rect: DOMRect) {
        // size of a maze in pixels
        let mazeSize = SNAILS[name].size * 20 * globalPos().scale;

        let leftOverflow = Math.floor(Math.max(-rect.left / mazeSize - 1, 0));
        let rightOverflow = Math.floor(Math.max((rect.right - window.innerWidth) / mazeSize - 1, 0));
        let topOverflow = Math.floor(Math.max(-rect.top / mazeSize - 1, 0));
        let bottomOverflow = Math.floor(Math.max((rect.bottom - window.innerHeight) / mazeSize - 1, 0));

        return { left: leftOverflow, right: rightOverflow, top: topOverflow, bottom: bottomOverflow };
    }

    // TODO: make async
    function updateLatticeMazes(name: SnailKey, visible: number[]) {
        const meshes = SNAILS[name].lattice.get_meshes(new Uint32Array(visible));

        for (let mesh of meshes) {
            SNAILS[name].mazes[mesh.id] = twgl.createBufferInfoFromArrays(gl, {
                position: { numComponents: 2, data: mesh.vertices },
                indices: { numComponents: 3, data: mesh.indices },
                instance_pos: { numComponents: 2, data: [0, 0], divisor: 1 },
            });
        }
    }

    let textures: Record<SnailKey, WebGLTexture> = {} as any;

    // initialize all lattices, and textures
    for (let name of SNAIL_NAMES) {
        let [snail, _setSnail] = SNAILS[name].store;
        SNAILS[name].lattice = new WasmLattice(name, randomSeed());

        createEffect((prevCount: number) => {
            let count = snail.count;
            let diff = count - prevCount;

            SNAILS[name].lattice.alter(diff);

            return count;
        }, 0);


        textures[name] = twgl.createTexture(gl, {
            src: `/assets/${name}.png`,
            min: gl.LINEAR,
            mag: gl.NEAREST,
            wrap: gl.CLAMP_TO_EDGE,
        });
    }

    textures["snail"] = twgl.createTexture(gl, {
        src: `/assets/snail.png`,
        min: gl.LINEAR,
        mag: gl.NEAREST,
        wrap: gl.CLAMP_TO_EDGE,
    });

    const tex = twgl.createTexture(gl, {
        min: gl.NEAREST,
        mag: gl.NEAREST,
        src: [
            0x06, 0x8f, 0xef, 0xff,
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

    let prevTime = performance.now();
    const renderContinuously = function(time: number) {
        let now = performance.now();

        for (let name of SNAIL_NAMES) {
            SNAILS[name].lattice.tick(now - prevTime);
        }

        render(time);
        requestAnimationFrame(renderContinuously);
        prevTime = now;
    };
    requestAnimationFrame(renderContinuously);
}

export function start() {
    init().then(() => {
        initEngine();
    });
}
