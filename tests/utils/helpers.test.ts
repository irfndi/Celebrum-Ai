/// <reference types="vitest/globals" />
import { describe, it, expect, vi } from 'vitest';
import {
  안전하게ParseFloat,
  깊은복제,
} from '../../src/utils/helpers';

describe('안전하게ParseFloat', () => {
  it('should parse valid numbers correctly', () => {
    expect(안전하게ParseFloat('123.45')).toBe(123.45);
    expect(안전하게ParseFloat('0')).toBe(0);
    expect(안전하게ParseFloat('-10.5')).toBe(-10.5);
    expect(안전하게ParseFloat(123)).toBe(123);
    expect(안전하게ParseFloat(123.456)).toBe(123.456);
  });

  it('should return defaultValue for invalid inputs', () => {
    expect(안전하게ParseFloat('abc')).toBe(0);
    expect(안전하게ParseFloat(null)).toBe(0);
    expect(안전하게ParseFloat(undefined)).toBe(0);
    expect(안전하게ParseFloat('')).toBe(0);
    expect(안전하게ParseFloat('  ')).toBe(0);
    expect(안전하게ParseFloat(Number.NaN)).toBe(0);
  });

  it('should return specified defaultValue for invalid inputs', () => {
    expect(안전하게ParseFloat('abc', 100)).toBe(100);
    expect(안전하게ParseFloat(null, -1)).toBe(-1);
    expect(안전하게ParseFloat(undefined, 99.9)).toBe(99.9);
  });

  it('should handle edge cases like Infinity and very large/small numbers if Number() supports them', () => {
    expect(안전하게ParseFloat(Number.POSITIVE_INFINITY)).toBe(Number.POSITIVE_INFINITY);
    expect(안전하게ParseFloat(Number.NEGATIVE_INFINITY)).toBe(Number.NEGATIVE_INFINITY);
    // Note: Behavior for extremely large/small strings might depend on JS Number limitations
  });
});

describe('깊은복제', () => {
  it('should clone primitive values', () => {
    expect(깊은복제(123)).toBe(123);
    expect(깊은복제('hello')).toBe('hello');
    expect(깊은복제(true)).toBe(true);
    expect(깊은복제(null)).toBe(null);
    expect(깊은복제(undefined)).toBe(undefined);
  });

  it('should deeply clone simple objects', () => {
    const obj = { a: 1, b: { c: 2 } };
    const clonedObj = 깊은복제(obj);
    expect(clonedObj).toEqual(obj);
    expect(clonedObj).not.toBe(obj);
    expect(clonedObj.b).not.toBe(obj.b);
  });

  it('should deeply clone arrays of primitives', () => {
    const arr = [1, 2, 3];
    const clonedArr = 깊은복제(arr);
    expect(clonedArr).toEqual(arr);
    expect(clonedArr).not.toBe(arr);
  });

  it('should deeply clone arrays of objects', () => {
    const arr = [{ a: 1 }, { b: 2 }];
    const clonedArr = 깊은복제(arr);
    expect(clonedArr).toEqual(arr);
    expect(clonedArr).not.toBe(arr);
    expect(clonedArr[0]).not.toBe(arr[0]);
  });

  it('should handle nested arrays and objects', () => {
    const complex = { arr: [1, { d: 4 }], obj: { e: { f: 5 } } };
    const clonedComplex = 깊은복제(complex);
    expect(clonedComplex).toEqual(complex);
    expect(clonedComplex.arr[1]).not.toBe(complex.arr[1]);
    expect(clonedComplex.obj.e).not.toBe(complex.obj.e);
  });

  it('should return a new object for empty objects and arrays', () => {
    const emptyObj = {};
    const clonedEmptyObj = 깊은복제(emptyObj);
    expect(clonedEmptyObj).toEqual(emptyObj);
    expect(clonedEmptyObj).not.toBe(emptyObj);

    const emptyArr: unknown[] = [];
    const clonedEmptyArr = 깊은복제(emptyArr);
    expect(clonedEmptyArr).toEqual(emptyArr);
    expect(clonedEmptyArr).not.toBe(emptyArr);
  });

  // Tests for limitations mentioned in the JSDoc of 깊은복제
  it('should convert Date objects to ISO strings', () => {
    const date = new Date();
    const cloned = 깊은복제({ d: date });
    expect(cloned.d).toBe(date.toISOString());
  });

  it('should remove undefined values from objects and convert to null in arrays', () => {
    const objWithUndefined = { a: 1, b: undefined, c: 3 };
    const clonedObjWithUndefined = 깊은복제(objWithUndefined);
    // JSON.stringify removes keys with undefined values
    expect(clonedObjWithUndefined).toEqual({ a: 1, c: 3 }); 

    const arrWithUndefined = [1, undefined, 3];
    const clonedArrWithUndefined = 깊은복제(arrWithUndefined);
    // JSON.stringify converts undefined in arrays to null
    expect(clonedArrWithUndefined).toEqual([1, null, 3]); 
  });

  it('should throw error for circular references if not handled by JSON.stringify', () => {
    const obj: Record<string, unknown> = { a: 1 };
    obj.circular = obj; // Create circular reference
    // Mock console.error to suppress expected error message during test
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => 깊은복제(obj)).toThrow('Failed to deep clone object due to its structure or content.');
    consoleErrorSpy.mockRestore();
  });
});
