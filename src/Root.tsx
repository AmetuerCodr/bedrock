import "./index.css";
import { CalculateMetadataFunction, Composition, staticFile } from "remotion";

import { Video, VideoProps } from "./Composition";
export const RemotionRoot: React.FC = () => {
  const func: CalculateMetadataFunction<VideoProps> = async () => {
    try {
      const res = await fetch(staticFile("data.json"));
      if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);
      const data = await res.json();
      const addedFrames = data.clipDurationInFrames.reduce(
        (acc: number, d: number) => acc + d,
        0,
      ); // combines duration of all frames
      console.log("data:", data);
      return {
        props: {
          script: data.wordGroups,
          clipDurationInFrames: data.clipDurationInFrames,
          displayFontArray: data.isDisplayFont,
          defaultTextVariant: data.defaultTextVariant,
        },
        durationInFrames: addedFrames,
      };
    } catch (err) {
      console.error("calculateMetadata error:", err);
      throw err;
    }
  };

  return (
    <>
      <Composition
        id="BedrockVideo"
        component={Video}
        defaultProps={{
          displayFontArray: [false, false, false],
          script: ["Preview", "text", "here"],
          defaultTextVariant: ["up", "down", "left"],
          clipDurationInFrames: [15, 15, 15],
        }}
        durationInFrames={450}
        calculateMetadata={func}
        fps={30}
        width={1080}
        height={1080}
      />
    </>
  );
};
