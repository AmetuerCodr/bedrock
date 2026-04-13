import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
} from "remotion";

import { loadFont as loadPoppins } from "@remotion/google-fonts/Poppins";
import { loadFont as loadBoldonse } from "@remotion/google-fonts/Boldonse";
import { getAvailableFonts } from "@remotion/google-fonts";
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
//
type MyCompProps = {
  fadeDirection: string;
  Text: string;
  styleBool: boolean;
  extraStyles: object;
  bodyFont: string;
  displayFont: string;
};

export const FastEaseText: React.FC<MyCompProps> = ({
  fadeDirection,
  Text,
  styleBool,
  extraStyles,
  bodyFont,
  displayFont,
}) => {
  const frame = useCurrentFrame();

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
        return ``;
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
    fontFamily?: string;
    transform?: string;
    textTransform?: string;
    opacity?: number;
  }

  const textStyleDefault: TextStyle = {
    color: "#fff",
    fontSize: 100,
    margin: 0,
    opacity,
  };

  const textStyleDisplay: TextStyle = {
    color: "#2563eb",
    fontSize: 100,
    margin: 0,
    textTransform: "uppercase",
    opacity,
  };

  let style: object = {};

  if (styleBool) {
    textStyleDisplay.transform = `${getFadeDirection(fadeDirection)}`;
    textStyleDefault.fontFamily = displayFont;
    style = textStyleDisplay;
  } else {
    textStyleDefault.transform = `${getFadeDirection(fadeDirection)}`;
    textStyleDefault.transform = bodyFont;
    style = textStyleDefault;
  }

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "#000",
      }}
    >
      <h1 style={{ ...style, ...extraStyles }}>{Text}</h1>
    </AbsoluteFill>
  );
};

// --- MAIN COMPOSITION WIRING ---
export type VideoProps = {
  script: string[];
  displayFontArray: boolean[];
  clipDurationInFrames: number[];
  defaultTextVariant: string[];
  fadeInTransitionBool: boolean[];
  bodyFont: string;
  displayFont: string;
};
export const Video: React.FC<VideoProps> = ({
  script,
  displayFontArray,
  clipDurationInFrames,
  defaultTextVariant,
  fadeInTransitionBool,
  bodyFont,
  displayFont,
}) => {
  displayFont = displayFont || "";
  bodyFont = bodyFont || "";
  const splitString = script || [];
  const fontArray = displayFontArray || [];
  const fadeInBool = fadeInTransitionBool || [];
  const animationVariants: string[] = defaultTextVariant || [];

  function opacityStyle(bool: boolean): object {
    let style = {};
    if (!bool) {
      style = { ...style, opacity: opacity };
      return style;
    } else {
      return {};
    }
  }

  const frame = useCurrentFrame();
  const easeOut = Easing.out(Easing.cubic);
  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });

  async function importFonts(fontName: string) {
    const availableFonts = getAvailableFonts();
    const font = availableFonts.find((f) => f.importName === fontName);

    if (!font) throw new Error(`Font ${fontName} not found`);

    const loaded = await font.load();
    loaded.loadFont("normal", {
      weights: ["400"],
      subsets: ["latin"],
    });
  }

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
              bodyFont={searchForFont("Inter")}
              displayFont={searchForFont("playfairdisplay")}
              styleBool={myStyleBool}
              extraStyles={opacityStyle(fadeInBool[i])}
              fadeDirection={fadeInBool[i] ? animationVariants[i] : "none"}
              Text={item}
            />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};

export default Video;
