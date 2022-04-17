# 使用说明

#### 整个工具包含两个部分：

1. vbf-tool.exe
2. 需要生成的vbf文件配置脚本，例如vbb.json

```bat
vbf-tool.exe <vbb.json>
```

#### vbb.json 说明

```json
{
    "VBF1":
    {
        "SourceFile": "./ti-proc.bin",
        "TargetFile": "./sample_bin.vbf",
        "VBFVersion": "2.6",
        "SwType": "EXE",
        "SwPartNum": "8894082291",
        "ECUaddr": "1432",
        "SwVersion": "A",
        "ImageOffset": "16777216",
        "CreateVerificationBlock": true,
        "VerificationBlockStartAddr": "4294960640",
        "Compressed": false,
        "Sort": true,
        "Group": true
    },
    "VBF2":
    {
        "tbd": "tbd"  
    }
}
```

![image-20220416011156915](C:\Users\67050\Desktop\ju_zi\image-20220416011156915.png)

#### profiling test

case1: 输入源文件989KB，消耗时间13ms

![image-20220416115559427](C:\d_vault\git_trace\vbf-tool\vbf-tool\vbf_tools\image-20220416115559427.png)

![image-20220416115726693](C:\d_vault\git_trace\vbf-tool\vbf-tool\vbf_tools\image-20220416115726693.png)

case2：输入源文件1.41GB，消耗时间12.5s

![image-20220416115928015](C:\d_vault\git_trace\vbf-tool\vbf-tool\vbf_tools\image-20220416115928015.png)

![image-20220416120112794](C:\d_vault\git_trace\vbf-tool\vbf-tool\vbf_tools\image-20220416120112794.png)

:star:**注意事项**：

- 当前版本只支持处理bin格式的源文件，所以需要提前将hex或者srec格式的文件导出到bin文件
- 该工具只能转换成VBF格式的文件，还需要配合吉利提供的签名工具生成带有签名的vbf文件