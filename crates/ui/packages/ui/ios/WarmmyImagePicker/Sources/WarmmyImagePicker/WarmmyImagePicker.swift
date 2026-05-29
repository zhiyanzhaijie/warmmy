import Foundation
import PhotosUI
import UIKit
import WebKit

@objc(WarmmyImagePicker)
public final class WarmmyImagePicker: NSObject, PHPickerViewControllerDelegate {
    private let maxImageEdge: CGFloat = 1536
    private let jpegQuality: CGFloat = 0.82
    private var webView: WKWebView?

    @objc public override init() {
        super.init()
    }

    @objc public func pickImages() {
        DispatchQueue.main.async {
            guard let controller = Self.topViewController() else {
                self.dispatch(files: [], error: "Unable to find iOS view controller")
                return
            }

            self.webView = Self.findWebView(in: controller.view)

            var config = PHPickerConfiguration(photoLibrary: .shared())
            config.filter = .images
            config.selectionLimit = 4

            let picker = PHPickerViewController(configuration: config)
            picker.delegate = self
            controller.present(picker, animated: true)
        }
    }

    @objc public func nativePickImages() {
        pickImages()
    }

    public func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
        picker.dismiss(animated: true)

        if results.isEmpty {
            dispatch(files: [], error: nil)
            return
        }

        let group = DispatchGroup()
        var files: [[String: Any]] = []
        var firstError: String?
        let lock = NSLock()

        for result in results {
            let provider = result.itemProvider
            guard provider.canLoadObject(ofClass: UIImage.self) else {
                continue
            }

            group.enter()
            provider.loadObject(ofClass: UIImage.self) { object, error in
                defer { group.leave() }

                if let error {
                    lock.lock()
                    if firstError == nil {
                        firstError = error.localizedDescription
                    }
                    lock.unlock()
                    return
                }

                guard let image = object as? UIImage else {
                    return
                }

                let derivedImage = self.resizeForWarmmy(image)
                guard let data = derivedImage.jpegData(compressionQuality: self.jpegQuality) else {
                    lock.lock()
                    if firstError == nil {
                        firstError = "Unable to encode selected image"
                    }
                    lock.unlock()
                    return
                }

                let name = provider.suggestedName.map { "\($0).jpg" } ?? "image.jpg"
                let dataUrl = "data:image/jpeg;base64,\(data.base64EncodedString())"
                let file: [String: Any] = [
                    "name": name,
                    "mime_type": "image/jpeg",
                    "size_bytes": data.count,
                    "data_url": dataUrl
                ]

                lock.lock()
                files.append(file)
                lock.unlock()
            }
        }

        group.notify(queue: .main) {
            self.dispatch(files: files, error: firstError)
        }
    }

    private func resizeForWarmmy(_ image: UIImage) -> UIImage {
        let width = image.size.width
        let height = image.size.height
        let longest = max(width, height)
        guard longest > maxImageEdge else {
            return image
        }

        let scale = maxImageEdge / longest
        let targetSize = CGSize(width: width * scale, height: height * scale)
        let renderer = UIGraphicsImageRenderer(size: targetSize)
        return renderer.image { _ in
            image.draw(in: CGRect(origin: .zero, size: targetSize))
        }
    }

    private func dispatch(files: [[String: Any]], error: String?) {
        var payload: [String: Any] = ["files": files]
        payload["error"] = error ?? NSNull()

        guard
            let data = try? JSONSerialization.data(withJSONObject: payload),
            let json = String(data: data, encoding: .utf8)
        else {
            return
        }

        let script = """
        window.dispatchEvent(new CustomEvent("warmmy-images-picked", {
            detail: \(json)
        }));
        """

        DispatchQueue.main.async {
            self.webView?.evaluateJavaScript(script)
        }
    }

    private static func topViewController() -> UIViewController? {
        let scene = UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .first { $0.activationState == .foregroundActive }

        let root = scene?.windows.first { $0.isKeyWindow }?.rootViewController
        return topViewController(from: root)
    }

    private static func topViewController(from controller: UIViewController?) -> UIViewController? {
        if let navigation = controller as? UINavigationController {
            return topViewController(from: navigation.visibleViewController)
        }
        if let tab = controller as? UITabBarController {
            return topViewController(from: tab.selectedViewController)
        }
        if let presented = controller?.presentedViewController {
            return topViewController(from: presented)
        }
        return controller
    }

    private static func findWebView(in view: UIView?) -> WKWebView? {
        guard let view else {
            return nil
        }
        if let webView = view as? WKWebView {
            return webView
        }
        for subview in view.subviews {
            if let webView = findWebView(in: subview) {
                return webView
            }
        }
        return nil
    }
}
