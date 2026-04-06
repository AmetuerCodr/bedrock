import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
  random,
} from "remotion";

import { loadFont as loadPoppins } from "@remotion/google-fonts/Poppins";
import { loadFont as loadBoldonse } from "@remotion/google-fonts/Boldonse";
import { useEffect } from "react";
const { fontFamily: Poppins } = loadPoppins("normal", {
  weights: ["400"],
  subsets: ["latin"],
});

const { fontFamily: Boldonse } = loadBoldonse("normal", {
  weights: ["400"],
  subsets: ["latin"],
});

// todo()! make the styling of text dynamic based on llm output

type MyCompProps = {
  fadeDirection: string;
  Text: string;
  styleBool: boolean;
};

export const FastEaseText: React.FC<MyCompProps> = ({
  fadeDirection,
  Text,
  styleBool,
}) => {
  const frame = useCurrentFrame();
  // FAST ease-out curve (snappy start, smooth finish)
  const easeOut = Easing.out(Easing.cubic);

  const getFadeDirection = (direction: string): string => {
    switch (direction) {
      case "left":
        return `translateX(${fade.left}px)`;
      case "right":
        return `translateX(${fade.right}px)`;
      case "top":
        return `translateY(${fade.top}px)`;
      case "bottom":
        return `translateY(${fade.bottom}px)`;
      default:
        return `translateX(${fade.left}px)`;
    }
  };

  const fade = Object.freeze({
    left: interpolate(frame, [0, 20], [-300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    right: interpolate(frame, [0, 20], [300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    top: interpolate(frame, [0, 20], [-300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    bottom: interpolate(frame, [0, 20], [300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
  });
  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });

  interface TextStyle {
    color: string;
    fontSize: number;
    margin?: number;
    fontFamily: string;
    transform?: string;
    textTransform?: string;
    opacity?: number;
  }

  const textStyleDefault: TextStyle = {
    color: "#fff",
    fontSize: 100,
    margin: 0,
    fontFamily: Poppins,
    transform: `${getFadeDirection(fadeDirection)}`,
    opacity,
  };

  const textStyleDisplay: TextStyle = {
    color: "#dc2626",
    fontSize: 100,
    margin: 0,
    textTransform: "uppercase",
    fontFamily: Boldonse,
    transform: `${getFadeDirection(fadeDirection)}`,
    opacity,
  };
  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "#000",
      }}
    >
      <h1 style={styleBool ? textStyleDisplay : textStyleDefault}>{Text}</h1>
    </AbsoluteFill>
  );
};

function randomFadeDirection(i: number) {
  const mynum = Math.floor(random(i) * 4);
  const directions = ["top", "bottom", "left", "right"];
  return directions[mynum];
}

// function myRegex(Text: string): string[] {
//   const regex = /\b[\w']+[^\s\w]*/g;
//   const splitString = Text?.match(regex) || [];
//   return splitString;
// }

// --- MAIN COMPOSITION WIRING ---
export type VideoProps = {
  script: string[];
  displayFontArray: boolean[];
  clipDurationInFrames: number[];
};
export const Video: React.FC<VideoProps> = ({
  script,
  displayFontArray,
  clipDurationInFrames,
}) => {
  // const splitString = myRegex("This Video Was Made With Code isn't that cool?");
  const splitString = script || [];
  const dur = clipDurationInFrames || [];
  const fontArray = displayFontArray || [];

  // useEffect(() => console.log(fontArray), []);

  return (
    <AbsoluteFill style={{ backgroundColor: "#0A0A0A" }}>
      {splitString.map((item, i) => {
        const fromFrame = clipDurationInFrames
          .slice(0, i)
          .reduce((acc, d) => acc + d, 0);

        const myStyleBool = fontArray[i];

        return (
          <Sequence
            key={i}
            durationInFrames={clipDurationInFrames[i]}
            from={fromFrame}
          >
            <FastEaseText
              styleBool={myStyleBool}
              fadeDirection={randomFadeDirection(i)}
              Text={item}
            />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};

export default Video;
