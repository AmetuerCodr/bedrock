import "./index.css";
import { CalculateMetadataFunction, Composition, staticFile } from "remotion";
import { Video } from "./Composition";
import { VideoSchema, VideoData } from "./lib/schema";

export const RemotionRoot: React.FC = () => {
  const func: CalculateMetadataFunction<VideoData> = async () => {
    const res = await fetch(staticFile("data.json"));
    if (!res.ok) throw new Error(`Failed to fetch: ${res.status}`);

    const data = VideoSchema.parse(await res.json()); // validates shape here

    return {
      props: data, // no manual remapping — field names already match
      durationInFrames: data.clipDurationInFrames.reduce((a, b) => a + b, 0),
    };
  };

  return (
    <Composition
      id="BedrockVideo"
      component={Video}
      calculateMetadata={func}
      defaultProps={{
        script: "",
        wordGroups: ["Preview", "text", "here"],
        TextPosition: "Center",
        clipDurationInFrames: [15, 15, 15],
        animationType: [["Fade", "Fade", "letterDrift"]],
        DisplayFontBoolArray: [[true, false, false]],
        displayFontColor: "#22d3ee",
        defaultTextVariant: ["left", "right", "bottom"],
        fadeInTransitionBool: [true, true, true],
        bodyFont: "inter",
        displayFont: "montserrat",
      }}
      durationInFrames={45}
      fps={30}
      width={1080}
      height={1080}
    />
  );
};
