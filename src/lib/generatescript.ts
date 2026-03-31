import { GoogleGenAI } from "@google/genai";
import { ElevenLabsClient } from "@elevenlabs/elevenlabs-js";

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
  const ai = new GoogleGenAI({
    apiKey: "AIzaSyADv-3Qbknvm4H4EpBtfOa0VSKTGJ1OvMM",
  });
  // const id = Bun.randomUUIDv7();
  // const file = Bun.file(`${id}.mp3`);
  // const writer = file.writer();
  const response = await ai.models.generateContent({
    model: "gemini-3-flash-preview",
    contents: prompt,
    config: {
      systemInstruction: `this script will be fed into a voice transcription api in it's entirety.
        you will be asked to produce some sort of script; when you do ensure that
        script only contains text no comments nothing in parenthesis or anything of the sort`,
    },
  });
  if (response.text) {
    Bun.write("script.txt", response.text);
    scriptToVoiceOver(response.text);
  }
}
generateScript("generate a super short script for a motivational tiktok.");
