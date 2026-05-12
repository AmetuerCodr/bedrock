import { ElevenLabsClient } from "@elevenlabs/elevenlabs-js";
export async function scriptToVoiceOver(text: string) {
  console.log("program running");
  if (!Bun.env.ELEVENLABS_VOICE_ID || !Bun.env.ELEVENLABS_KEY) return;
  const elevenlabsclient = new ElevenLabsClient({
    apiKey: Bun.env.ELEVENLABS_KEY,
  });
  const id = Bun.randomUUIDv7();
  const file = Bun.file(`${id}.mp3`);
  const writer = file.writer();
  const audio: ReadableStream = await elevenlabsclient.textToSpeech.convert(
    Bun.env.ELEVENLABS_VOICE_ID,
    // allow the voice to be dynamic
    // figure out how to sync the voice to the words and animation
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
