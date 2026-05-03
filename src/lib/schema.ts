// schema.ts
import { z } from "zod";

export const FadeDirection = z.enum(["top", "bottom", "left", "right"]);
export const Animation = z.enum(["Fade", "letterDrift", "shearSnap"]);
export const TextPosition = z.enum(["Top", "Center", "Bottom"]);

export const VideoSchema = z.object({
  script: z.string(),
  wordGroups: z.array(z.string()),
  clipDurationInFrames: z.array(z.number().int().positive()),
  DisplayFontBoolArray: z.array(z.array(z.boolean())),
  defaultTextVariant: z.array(FadeDirection),
  animationType: z.array(z.array(Animation)),
  fadeInTransitionBool: z.array(z.boolean()),
  bodyFont: z.string(),
  displayFont: z.string(),
  displayFontColor: z.string(),
  TextPosition: TextPosition,
});

export type VideoData = z.infer<typeof VideoSchema>;
