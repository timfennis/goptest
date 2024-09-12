**goptest** is a simple cli utility that uses ffmpeg to to check the keyframe interval of a video. For livestreaming you often want the GOP-size of your video's to be small and consisten all the way through.

## Usage

```bash
goptest some-video.mp4
```

Might output something like

```
Stream 0 has a consistent GOP size of 48
```
