/* tslint:disable */
/* eslint-disable */
/**
 * Initialize panic hook for better error messages in the browser
 */
export function init(): void;
/**
 * Tokenize Mica source code
 */
export function tokenize(source: string): string;
/**
 * Parse Mica source code into AST
 */
export function parse_ast(source: string, pretty: boolean): string;
/**
 * Resolve names and check capabilities
 */
export function resolve_code(source: string): string;
/**
 * Check exhaustiveness and effects
 */
export function check_code(source: string): string;
/**
 * Lower to HIR
 */
export function lower_code(source: string): string;
/**
 * Generate IR
 */
export function generate_ir(source: string): string;
/**
 * Run Mica code
 */
export function run_code(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init: () => void;
  readonly tokenize: (a: number, b: number) => [number, number];
  readonly parse_ast: (a: number, b: number, c: number) => [number, number];
  readonly resolve_code: (a: number, b: number) => [number, number];
  readonly check_code: (a: number, b: number) => [number, number];
  readonly lower_code: (a: number, b: number) => [number, number];
  readonly generate_ir: (a: number, b: number) => [number, number];
  readonly run_code: (a: number, b: number) => [number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
