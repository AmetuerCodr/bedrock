// schema.ts
import { z } from "zod";

export const FadeDirection = z.enum(["top", "bottom", "left", "right"]);

export const VideoSchema = z.object({
  script: z.string(),
  wordGroups: z.array(z.string()),
  clipDurationInFrames: z.array(z.number().int().positive()),
  DisplayFontBoolArray: z.array(z.array(z.boolean())),
  defaultTextVariant: z.array(FadeDirection),
  fadeInTransitionBool: z.array(z.boolean()),
  bodyFont: z.string(),
  displayFont: z.string(),
  displayFontColor: z.string(),
});

export type VideoData = z.infer<typeof VideoSchema>;
