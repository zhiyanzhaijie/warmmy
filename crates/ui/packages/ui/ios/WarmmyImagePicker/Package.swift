// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "WarmmyImagePicker",
    platforms: [
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "WarmmyImagePicker",
            type: .dynamic,
            targets: ["WarmmyImagePicker"]
        )
    ],
    targets: [
        .target(
            name: "WarmmyImagePicker",
            dependencies: []
        )
    ]
)

