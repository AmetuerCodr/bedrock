import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
} from "remotion";
import { getAvailableFonts } from "@remotion/google-fonts";
import React from "react";
import { VideoData } from "./lib/schema.ts";

import { useState, useEffect } from "react";

// Dynamic font loader — now actually wired up
async function loadFont(importName: string): Promise<string> {
  const available = getAvailableFonts();
  const font = available.find((f) => f.importName.toLowerCase() === importName.toLowerCase());
  if (!font) throw new Error(`Font not found: ${importName}`);
  const loaded = await font.load();
  const { fontFamily } = loaded.loadFont("normal", {
    weights: ["400"],
    subsets: ["latin"],
  });
  return fontFamily;
}

// --- Clip component ---
type ClipProps = {
  text: string;
  isDisplay: boolean;
  fadeDir: string;
  hasFade: boolean;
  fontFamily: string;
};

const Clip: React.FC<ClipProps> = ({
  text,
  isDisplay,
  fadeDir,
  hasFade,
  fontFamily,
}) => {
  const frame = useCurrentFrame();
  const easeOut = Easing.out(Easing.cubic);

  const offset = interpolate(
    frame,
    [0, 20],
    [
      fadeDir === "left"
        ? -300
        : fadeDir === "right"
          ? 300
          : fadeDir === "top"
            ? -300
            : 300,
      0,
    ],
    { easing: easeOut, extrapolateRight: "clamp" },
  );

  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });

  const translate =
    fadeDir === "left" || fadeDir === "right"
      ? `translateX(${offset}px)`
      : `translateY(${offset}px)`;

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "#000",
      }}
    >
      <h1
        style={{
          color: isDisplay ? "#2563eb" : "#fff",
          fontSize: 100,
          margin: 0,
          fontFamily,
          textTransform: isDisplay ? "uppercase" : undefined,
          transform: hasFade ? translate : undefined,
          opacity: hasFade ? opacity : 1,
        }}
      >
        {text}
      </h1>
    </AbsoluteFill>
  );
};

// --- Main Video component — props are now just VideoData ---
export const Video: React.FC<VideoData> = ({
  wordGroups,
  clipDurationInFrames,
  DisplayFontBoolArray,
  defaultTextVariant,
  fadeInTransitionBool,
  bodyFont,
  displayFont,
}) => {
  const [bodyFamily, setBodyFamily] = useState("sans-serif");
  const [displayFamily, setDisplayFamily] = useState("serif");

  useEffect(() => {
    loadFont(bodyFont).then(setBodyFamily);
    loadFont(displayFont).then(setDisplayFamily);
  }, [bodyFont, displayFont]);

  let fromFrame = 0;

  return (
    <AbsoluteFill style={{ backgroundColor: "#0A0A0A" }}>
      {wordGroups.map((text, i) => {
        const start = fromFrame;
        fromFrame += clipDurationInFrames[i];

        return (
          <Sequence
            key={i}
            from={start}
            durationInFrames={clipDurationInFrames[i]}
          >
            <Clip
              text={text}
              isDisplay={DisplayFontBoolArray[i]}
              fadeDir={defaultTextVariant[i]}
              hasFade={fadeInTransitionBool[i]}
              fontFamily={DisplayFontBoolArray[i] ? displayFamily : bodyFamily}
            />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};

export default Video;
