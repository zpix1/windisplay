type IconProps = {
  size?: number;
};

export function RotateLeftIcon({ size = 18 }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
      <g
        id="SVGRepo_tracerCarrier"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></g>
      <g id="SVGRepo_iconCarrier">
        {" "}
        <path
          d="M3.51018 14.9907C4.15862 16.831 5.38765 18.4108 7.01208 19.492C8.63652 20.5732 10.5684 21.0972 12.5165 20.9851C14.4647 20.873 16.3237 20.1308 17.8133 18.8704C19.303 17.61 20.3426 15.8996 20.7756 13.997C21.2086 12.0944 21.0115 10.1026 20.214 8.32177C19.4165 6.54091 18.0617 5.06746 16.3539 4.12343C14.6461 3.17941 12.6777 2.81593 10.7454 3.08779C7.48292 3.54676 5.32746 5.91142 3 8M3 8V2M3 8H9"
          stroke="#000000"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        ></path>{" "}
      </g>
    </svg>
  );
}

export function RotateRightIcon({ size = 18 }: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
      <g
        id="SVGRepo_tracerCarrier"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></g>
      <g id="SVGRepo_iconCarrier">
        {" "}
        <path
          d="M20.4898 14.9907C19.8414 16.831 18.6124 18.4108 16.9879 19.492C15.3635 20.5732 13.4316 21.0972 11.4835 20.9851C9.5353 20.873 7.67634 20.1308 6.18668 18.8704C4.69703 17.61 3.65738 15.8996 3.22438 13.997C2.79138 12.0944 2.98849 10.1026 3.78602 8.32177C4.58354 6.54091 5.93827 5.06746 7.64608 4.12343C9.35389 3.17941 11.3223 2.81593 13.2546 3.08779C16.5171 3.54676 18.6725 5.91142 21 8M21 8V2M21 8H15"
          stroke="#000000"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        ></path>{" "}
      </g>
    </svg>
  );
}

type MirrorIconProps = {
  size?: number;
  rotated?: boolean;
};

export function MirrorIcon({ size = 18, rotated = false }: MirrorIconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke="#000000"
      style={rotated ? { transform: "rotate(90deg)" } : undefined}
    >
      <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
      <g
        id="SVGRepo_tracerCarrier"
        stroke-linecap="round"
        stroke-linejoin="round"
      ></g>
      <g id="SVGRepo_iconCarrier">
        {" "}
        <path
          d="M2 8.00024V5.88641C2 4.18426 2 3.33319 2.54242 3.05405C3.08484 2.77491 3.77738 3.26959 5.16247 4.25894L6.74371 5.3884C7.35957 5.8283 7.6675 6.04825 7.83375 6.3713C8 6.69435 8 7.07277 8 7.8296V16.1705C8 16.9273 8 17.3057 7.83375 17.6288C7.6675 17.9518 7.35957 18.1718 6.74372 18.6117L5.16248 19.7411C3.77738 20.7305 3.08484 21.2251 2.54242 20.946C2 20.6669 2 19.8158 2 18.1136V12.0002"
          stroke="#000000"
          stroke-width="1.5"
          stroke-linecap="round"
        ></path>{" "}
        <path
          d="M22 12.0002V5.88641C22 4.18426 22 3.33319 21.4576 3.05405C20.9152 2.77491 20.2226 3.26959 18.8375 4.25894L17.2563 5.3884C16.6404 5.8283 16.3325 6.04825 16.1662 6.3713C16 6.69435 16 7.07277 16 7.8296V16.1705C16 16.9273 16 17.3057 16.1662 17.6288C16.3325 17.9518 16.6404 18.1718 17.2563 18.6117L18.8375 19.7411C20.2226 20.7305 20.9152 21.2251 21.4576 20.946C22 20.6669 22 19.8158 22 18.1136V16.0569"
          stroke="#000000"
          stroke-width="1.5"
          stroke-linecap="round"
        ></path>{" "}
        <path
          d="M12 14V10M12 6V2M12 22V18"
          stroke="#000000"
          stroke-width="1.5"
          stroke-linecap="round"
        ></path>{" "}
      </g>
    </svg>
  );
}
