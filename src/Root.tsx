import "./index.css";
import { Composition } from "remotion";
import Video from "./Composition";
export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="KineticTypography"
        component={Video}
        defaultProps={{ script: ["Preview", "text", "here"] }}
        durationInFrames={450}
        fps={30}
        width={1080}
        height={1080}
      />
    </>
  );
};
