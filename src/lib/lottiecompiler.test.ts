import { describe, expect, test } from "bun:test";
import { VisualMomentArraySchema, VisualMomentSchema } from "./schema";

describe("VisualMomentArraySchema (LLM output validation)", () => {
  test("accepts a known-good single-element fixture", () => {
    const fixture = [
      {
        index: 0,
        concept: "growth",
        mood: "hopeful",
        metaphor: "expanding circle",
        duration: 30,
      },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).not.toThrow();
  });

  test("accepts a multi-element fixture with monotonically increasing indices", () => {
    const fixture = [
      { index: 0, concept: "purpose", mood: "hopeful", metaphor: "rising horizon line", duration: 35 },
      { index: 6, concept: "friction", mood: "tense", metaphor: "converging arcs", duration: 30 },
      { index: 13, concept: "grit", mood: "somber", metaphor: "shifting earth", duration: 28 },
      { index: 27, concept: "arrival", mood: "triumphant", metaphor: "expanding circle", duration: 40 },
    ];
    const parsed = VisualMomentArraySchema.parse(fixture);
    expect(parsed).toHaveLength(4);
    expect(parsed[3].mood).toBe("triumphant");
  });

  test("rejects mood outside the closed enum", () => {
    const fixture = [
      { index: 0, concept: "joy", mood: "excited", metaphor: "burst of light", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects duration below 15", () => {
    const fixture = [
      { index: 0, concept: "growth", mood: "hopeful", metaphor: "expanding circle", duration: 5 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects duration above 90", () => {
    const fixture = [
      { index: 0, concept: "growth", mood: "hopeful", metaphor: "expanding circle", duration: 200 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects negative index", () => {
    const fixture = [
      { index: -1, concept: "growth", mood: "hopeful", metaphor: "expanding circle", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects non-integer index", () => {
    const fixture = [
      { index: 1.5, concept: "growth", mood: "hopeful", metaphor: "expanding circle", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects missing concept field", () => {
    const fixture = [
      { index: 0, mood: "hopeful", metaphor: "expanding circle", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects empty-string concept", () => {
    const fixture = [
      { index: 0, concept: "", mood: "hopeful", metaphor: "expanding circle", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects empty-string metaphor", () => {
    const fixture = [
      { index: 0, concept: "growth", mood: "hopeful", metaphor: "", duration: 30 },
    ];
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects an object where an array is expected", () => {
    const fixture = {
      index: 0,
      concept: "growth",
      mood: "hopeful",
      metaphor: "expanding circle",
      duration: 30,
    };
    expect(() => VisualMomentArraySchema.parse(fixture)).toThrow();
  });

  test("rejects malformed JSON (simulated via JSON.parse + schema)", () => {
    const llmOutput = "not json at all";
    expect(() => JSON.parse(llmOutput)).toThrow();
  });

  test("rejects when LLM returns an empty string", () => {
    expect(() => JSON.parse("")).toThrow();
  });

  test("accepts every valid mood value individually", () => {
    const moods = [
      "energetic",
      "calm",
      "tense",
      "hopeful",
      "somber",
      "playful",
      "ominous",
      "triumphant",
    ];
    for (const mood of moods) {
      const fixture = {
        index: 0,
        concept: "growth",
        mood,
        metaphor: "expanding circle",
        duration: 30,
      };
      expect(() => VisualMomentSchema.parse(fixture)).not.toThrow();
    }
  });

  test("accepts boundary duration values 15 and 90", () => {
    const low = {
      index: 0,
      concept: "growth",
      mood: "hopeful",
      metaphor: "expanding circle",
      duration: 15,
    };
    const high = {
      index: 0,
      concept: "growth",
      mood: "hopeful",
      metaphor: "expanding circle",
      duration: 90,
    };
    expect(() => VisualMomentSchema.parse(low)).not.toThrow();
    expect(() => VisualMomentSchema.parse(high)).not.toThrow();
  });
});
