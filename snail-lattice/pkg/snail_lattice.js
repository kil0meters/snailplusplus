
let wasm;

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

const cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = new Uint8Array();

function getUint8Memory0() {
    if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let cachedInt32Memory0 = new Int32Array();

function getInt32Memory0() {
    if (cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let cachedUint32Memory0 = new Uint32Array();

function getUint32Memory0() {
    if (cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayU32FromWasm0(ptr, len) {
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}
/**
*/
export class CloneLattice {

    static __wrap(ptr) {
        const obj = Object.create(CloneLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_clonelattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.clonelattice_new(width, seed);
        return CloneLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.clonelattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.clonelattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.clonelattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.clonelattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class HoldLeftLattice {

    static __wrap(ptr) {
        const obj = Object.create(HoldLeftLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_holdleftlattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.clonelattice_new(width, seed);
        return HoldLeftLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.holdleftlattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.holdleftlattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.holdleftlattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.holdleftlattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class LearningLattice {

    static __wrap(ptr) {
        const obj = Object.create(LearningLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_learninglattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.clonelattice_new(width, seed);
        return LearningLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.holdleftlattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.learninglattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.learninglattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.learninglattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class RandomTeleportLattice {

    static __wrap(ptr) {
        const obj = Object.create(RandomTeleportLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_randomteleportlattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.clonelattice_new(width, seed);
        return RandomTeleportLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.randomteleportlattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.randomteleportlattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.randomteleportlattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.randomteleportlattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class RandomWalkLattice {

    static __wrap(ptr) {
        const obj = Object.create(RandomWalkLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_randomwalklattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.clonelattice_new(width, seed);
        return RandomWalkLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.randomwalklattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.randomwalklattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.randomwalklattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.randomwalklattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class TimeTravelLattice {

    static __wrap(ptr) {
        const obj = Object.create(TimeTravelLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_timetravellattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.timetravellattice_new(width, seed);
        return TimeTravelLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.timetravellattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.timetravellattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.timetravellattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.timetravellattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}
/**
*/
export class TremauxLattice {

    static __wrap(ptr) {
        const obj = Object.create(TremauxLattice.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tremauxlattice_free(ptr);
    }
    /**
    * @param {number} width
    * @param {number} seed
    */
    constructor(width, seed) {
        const ret = wasm.timetravellattice_new(width, seed);
        return TremauxLattice.__wrap(ret);
    }
    /**
    * @param {number} count
    * @returns {Uint32Array}
    */
    get_dimensions(count) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.tremauxlattice_get_dimensions(retptr, this.ptr, count);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {Uint8Array} buffer
    * @param {number} index
    * @param {number} count
    */
    render(buffer, index, count) {
        try {
            var ptr0 = passArray8ToWasm0(buffer, wasm.__wbindgen_malloc);
            var len0 = WASM_VECTOR_LEN;
            wasm.tremauxlattice_render(this.ptr, ptr0, len0, index, count);
        } finally {
            buffer.set(getUint8Memory0().subarray(ptr0 / 1, ptr0 / 1 + len0));
            wasm.__wbindgen_free(ptr0, len0 * 1);
        }
    }
    /**
    * @param {number} dt
    * @returns {number}
    */
    tick(dt) {
        const ret = wasm.tremauxlattice_tick(this.ptr, dt);
        return ret >>> 0;
    }
    /**
    * @param {number} difference
    */
    alter(difference) {
        wasm.tremauxlattice_alter(this.ptr, difference);
    }
    /**
    * @returns {number}
    */
    count() {
        const ret = wasm.clonelattice_count(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} width
    */
    set_width(width) {
        wasm.clonelattice_set_width(this.ptr, width);
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function getImports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
        const ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
        try {
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(arg0, arg1);
        }
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function initMemory(imports, maybe_memory) {

}

function finalizeInit(instance, module) {
    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    cachedInt32Memory0 = new Int32Array();
    cachedUint32Memory0 = new Uint32Array();
    cachedUint8Memory0 = new Uint8Array();


    return wasm;
}

function initSync(module) {
    const imports = getImports();

    initMemory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return finalizeInit(instance, module);
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = new URL('snail_lattice_bg.wasm', import.meta.url);
    }
    const imports = getImports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    initMemory(imports);

    const { instance, module } = await load(await input, imports);

    return finalizeInit(instance, module);
}

export { initSync }
export default init;
