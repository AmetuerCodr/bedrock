import { GoogleGenAI } from "@google/genai";
import { ElevenLabsClient } from "@elevenlabs/elevenlabs-js";
import { parseArgs } from "util";

async function scriptToVoiceOver(text: string) {
  console.log("program running");
  const elevenlabsclient = new ElevenLabsClient({
    apiKey: Bun.env.ELEVENLABS_KEY,
  });
  const id = Bun.randomUUIDv7();
  const file = Bun.file(`${id}.mp3`);
  const writer = file.writer();
  const audio: ReadableStream = await elevenlabsclient.textToSpeech.convert(
    "pqHfZKP75CvOlQylNhV4",
    {
      outputFormat: "mp3_44100_128",
      text: text,
      modelId: "eleven_multilingual_v2",
    },
  );

  if (audio) {
    const buf = await new Response(audio as ReadableStream).arrayBuffer();
    writer.write(buf);
    console.log("voiceover written to disk!");
  }
}

export async function generateScript(prompt: string) {
  //todo make the api swap models when in high demand

  // 13061 |         const errorMessage = JSON.stringify(errorBody);
  // 13062 |         if (status >= 400 && status < 600) {
  // 13063 |             const apiError = new ApiError({
  //                                      ^
  // ApiError: {"error":{"code":503,"message":"This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.","status":"UNAVAILABLE"}}
  //  status: 503,
  const ai = new GoogleGenAI({
    apiKey: Bun.env.GEMINI_API_KEY,
  });
  const response = await ai.models.generateContent({
    model: "gemini-2.5-flash",
    contents: prompt,
    config: {
      systemInstruction: `give no extra commentary, all you are to produce is some json output with the following structure:
      {
      "script": "[here is an example script] People will forget what you said... but people will never forget how you made them feel,"
      "regExScript:" "

      Your task here is to break a the script you made into a JSON array of strings based on these rules:

      1. STRUCTURE: Elements must alternate between exactly TWO words and exactly ONE word.
      2. EMPHASIS: Single-word elements must be the "anchor" words—verbs, nouns, or adjectives that carry the most meaning.
      3. FLOW: Two-word elements should contain "functional" words (the, is, with, they).
      4. FORMAT: Output ONLY a valid JSON array of strings.

      Example:
      Input: "The quick brown fox jumps over the lazy dog"
      Output: ["The quick", "brown", "fox jumps", "over", "the lazy", "dog"]

      based on our example script quote here is what the output would look like:
      [
      "People will",
      "forget",
      "what you",
      "said...",
      "but people",
      "will never",
      "forget",
      "how you",
      "made them",
      "feel"
      ]
      "
      }
      here is the description of the script i want you to write:
      `,
    },
  });
  if (response.text) {
    const cleanedContents = response.text
      .replace(/^```json\s*/i, "")
      .replace(/```\s*$/i, "")
      .trim();
    Bun.write("./public/data.json", cleanedContents);
    // scriptToVoiceOver(response.text);
  }
}
const { values } = parseArgs({
  args: Bun.argv,
  options: {
    m: {
      type: "string",
    },
  },
  strict: true,
  allowPositionals: true,
});
const message = values.m?.toString();
if (message) generateScript(message);
// systemInstruction: `this script will be fed into a voice transcription api in it's entirety.
//   you will be asked to produce some sort of script; when you do ensure that
//   script only contains text no comments nothing in parenthesis or anything of the sort`,
// },
