import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
  useVideoConfig,
} from "remotion";
import { getAvailableFonts } from "@remotion/google-fonts";
import React from "react";
import { useState, useEffect } from "react";
import { VideoData } from "./lib/schema.ts";
import { shearSnap } from "./lib/animations/shearsnap.ts";
import { letterDrift } from "./lib/animations/letterdrift.ts";

let animationStyle;
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
  animationType: string[];
  text: string;
  fontBools: boolean[]; // Replaces isDisplay
  fadeDir: string;
  hasFade: boolean;
  displayFamily: string; // The bold font
  bodyFamily: string; // The clean font
  color: string;
  wordGroups: string[];
}

/// animations

// animations/letterDrift.ts

export const Clip: React.FC<ClipProps> = ({
  text,
  fontBools,
  fadeDir,
  hasFade,
  wordGroups,
  displayFamily,
  bodyFamily,
  animationType,
  color,
}) => {
  const frame = useCurrentFrame();
  const easeOut = Easing.out(Easing.cubic);

  const { fps } = useVideoConfig();

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

  // console.log(animationType);
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
        }}
      >
        {/* Map over the words and style them individually based on the fontBools array */}
        {words.map((word, index) => {
          // Safety check in case the LLM messes up the array length

          // console.log(wordGroups);
          const isDisplay = fontBools[index] ?? false;
          const a = animationType[index];

          const { x, y, rotate, opacity } = letterDrift(frame, fps, index);
          const { skewX, translateX, blur } = shearSnap(frame, fps, index, {
            delayPerWord: 8,
            damping: 26,
            stiffness: 70,
          });

          // fade && letter drift  && shearSnap

          const letterDriftStyle = {
            transform: `translate(${x}px, ${y}px) rotate(${rotate}deg)`,
            opacity: opacity,
          };

          const shearSnapStyle = {
            transform: `skewX(${skewX}deg) translateX(${translateX}px)`,
            filter: `blur(${blur}px)`,
          };

          const fadeStyle = {
            transform: translate,
            opacity: opacity,
          };

          function playAnimation(animation: string): void {
            switch (animation) {
              case "letterDrift":
                animationStyle = letterDriftStyle;

                console.log(
                  `animation type: ${animationType[index]} at index ${index}`,
                );

                break;
              case "shearSnap":
                animationStyle = shearSnapStyle;

                console.log(
                  `animation type: ${animationType[index]} at index ${index}`,
                );
                break;
              case "Fade":
                animationStyle = fadeStyle;

                console.log(
                  `animation type: ${animationType[index]} at index ${index}`,
                );
                break;
            }
          }

          playAnimation(a);

          return (
            <span
              key={index}
              style={{
                ...animationStyle,
                color: isDisplay ? color : "#fff",
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
  animationType,
  bodyFont,
  displayFont,
  displayFontColor,
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
              wordGroups={wordGroups}
              text={text}
              animationType={animationType[i]}
              // Pass the sub-array of booleans for this specific word group

              // color={
              fontBools={DisplayFontBoolArray[i]}
              fadeDir={defaultTextVariant[i]}
              hasFade={fadeInTransitionBool[i]}
              color={displayFontColor}
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
