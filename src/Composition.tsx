import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
} from "remotion";
import { getAvailableFonts } from "@remotion/google-fonts";
import React from "react";
import { useState, useEffect } from "react";
import { VideoData } from "./lib/schema.ts";


// Dynamic font loader 
async function loadFont(importName: string): Promise<string> {
  const available = getAvailableFonts();
  const font = available.find(
    (f) => f.importName.toLowerCase() === importName.toLowerCase(),
  );
  if (!font) throw new Error(`Font not found: ${importName}`);
  const loaded = await font.load();
  const { fontFamily } = loaded.loadFont("normal", {
    weights: ["400"],
    subsets: ["latin"],
  });
  return fontFamily;
}

// 1. Update your interface/types for the new props
interface ClipProps {
  text: string;
  fontBools: boolean[]; // Replaces isDisplay
  fadeDir: string;
  hasFade: boolean;
  displayFamily: string; // The bold font
  bodyFamily: string;    // The clean font
}

export const Clip: React.FC<ClipProps> = ({
  text,
  fontBools,
  fadeDir,
  hasFade,
  displayFamily,
  bodyFamily,
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
    { easing: easeOut, extrapolateRight: "clamp" }
  );

  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });

  const translate =
    fadeDir === "left" || fadeDir === "right"
      ? `translateX(${offset}px)`
      : `translateY(${offset}px)`;

  // Split the chunk of text into an array of words
  const words = text.split(" ");

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "#000",
      }}
    >
      {/* The Wrapper handles the animation and layout so the words move together */}
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          flexWrap: "wrap",
          justifyContent: "center",
          gap: "25px", // Acts as your "space" between words. Adjust as needed!
          fontSize: 100,
          margin: 0,
          transform: hasFade ? translate : undefined,
          opacity: hasFade ? opacity : 1,
        }}
      >
        {/* Map over the words and style them individually based on the fontBools array */}
        {words.map((word, index) => {
          // Safety check in case the LLM messes up the array length
          const isDisplay = fontBools[index] ?? false;

          return (
            <span
              key={index}
              style={{
                color: isDisplay ? "#f59e0b" : "#fff",
                fontFamily: isDisplay ? displayFamily : bodyFamily,
                textTransform: isDisplay ? "capitalize" : undefined,
                fontWeight: isDisplay ? 700 : 100,
              }}
            >
              {word}
            </span>
          );
        })}
      </div>
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
              // Pass the sub-array of booleans for this specific word group
              fontBools={DisplayFontBoolArray[i]} 
              fadeDir={defaultTextVariant[i]}
              hasFade={fadeInTransitionBool[i]}
              // Pass BOTH fonts so the Clip component can alternate them
              displayFamily={displayFamily}
              bodyFamily={bodyFamily}
            />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};

export default Video;
